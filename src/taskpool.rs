use crate::error::{HackshellError, HackshellResult, JoinError};

#[cfg(feature = "async")]
use std::future::Future;

use std::{
    any::Any,
    collections::HashMap,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::JoinHandle,
};

#[derive(Clone)]
pub struct TaskMetadata {
    pub name: String,
    pub started: chrono::DateTime<chrono::Utc>,
    pub id: u64,
}

pub type TaskOutput = Option<Box<dyn Any + Send>>;

enum TaskInner {
    Sync {
        /// This is used to signal a sync thread to stop gracefully.
        /// In Rust, due to memory safety, it's not possible to stop normal threads, as they have no
        /// yielding points.
        run: Mutex<Option<Arc<AtomicBool>>>,
        join_handle: Mutex<Option<JoinHandle<TaskOutput>>>,
    },

    #[cfg(feature = "async")]
    Async {
        join_handle: Mutex<Option<tokio::task::JoinHandle<TaskOutput>>>,
    },
}

struct Task {
    meta: TaskMetadata,
    inner: TaskInner,
}

impl Task {
    fn meta(&self) -> TaskMetadata {
        self.meta.clone()
    }

    fn kill(&self) -> HackshellResult<()> {
        match &self.inner {
            TaskInner::Sync { run, .. } => {
                if let Some(run) = run.lock().unwrap().take() {
                    run.store(false, Ordering::Relaxed);
                }
            }
            #[cfg(feature = "async")]
            TaskInner::Async { join_handle } => {
                if let Some(handle) = join_handle.lock().unwrap().take() {
                    handle.abort();
                }
            }
        }
        Ok(())
    }

    fn join(&self) -> HackshellResult<TaskOutput> {
        match &self.inner {
            TaskInner::Sync { join_handle, .. } => {
                let wh = join_handle
                    .lock()
                    .unwrap()
                    .take()
                    .ok_or(HackshellError::JoinError(JoinError::AlreadyJoining))?;

                wh.join().map_err(|e| {
                    HackshellError::JoinError(JoinError::Sync(Box::new(Mutex::new(e))))
                })
            }

            #[cfg(feature = "async")]
            TaskInner::Async { join_handle } => {
                let wh = join_handle
                    .lock()
                    .unwrap()
                    .take()
                    .ok_or(HackshellError::JoinError(JoinError::AlreadyJoining))?;

                let handle = tokio::runtime::Handle::try_current().map_err(|e| e.to_string())?;

                handle.block_on(async move {
                    wh.await
                        .map_err(|e| HackshellError::JoinError(JoinError::Async(e)))
                })
            }
        }
    }

    #[cfg(feature = "async")]
    async fn join_async(&self) -> HackshellResult<TaskOutput> {
        match &self.inner {
            TaskInner::Sync { .. } => Err(HackshellError::JoinError(JoinError::CannotJoinAsync)),
            TaskInner::Async { join_handle } => {
                let wh = join_handle
                    .lock()
                    .unwrap()
                    .take()
                    .ok_or(HackshellError::JoinError(JoinError::AlreadyJoining))?;

                wh.await
                    .map_err(|e| HackshellError::JoinError(JoinError::Async(e)))
            }
        }
    }
}

#[derive(Default)]
struct InnerTaskPool {
    task_id: Arc<AtomicU64>,
    tasks: RwLock<HashMap<String, Task>>,
}

impl InnerTaskPool {
    fn kill_all(&self) {
        let tasks: Vec<Task> = self
            .tasks
            .write()
            .unwrap()
            .drain()
            .map(|(_, t)| t)
            .collect();

        for task in tasks {
            let _ = task.kill();
        }
    }

    fn remove_by_id(&self, id: u64) -> HackshellResult<()> {
        let mut tasks = self.tasks.write().unwrap();

        let key = tasks
            .iter()
            .find(|(_, v)| v.meta().id == id)
            .map(|(k, _)| k.clone())
            .ok_or(HackshellError::TaskNotFound)?;

        let (_, task) = tasks.remove_entry(&key).unwrap();

        task.kill()?;

        Ok(())
    }
}

impl Drop for InnerTaskPool {
    fn drop(&mut self) {
        self.kill_all()
    }
}

#[derive(Default, Clone)]
pub struct TaskPool {
    inner: Arc<InnerTaskPool>,
}

impl TaskPool {
    fn gen_task_id(&self) -> u64 {
        self.inner.task_id.fetch_add(1, Ordering::Relaxed)
    }

    pub fn spawn<F>(&self, name: &str, func: F)
    where
        F: FnOnce(Arc<AtomicBool>) -> TaskOutput + Send + 'static,
    {
        let run = Arc::new(AtomicBool::new(true));
        let run_ref = run.clone();
        let weak_inner = Arc::downgrade(&self.inner);
        let name = name.to_string();

        // There could or could not be the task with the same name.
        // In the case it's there, we kill it and insert the new one.
        let _ = self.remove(&name);
        let id = self.gen_task_id();

        let handle = std::thread::spawn(move || {
            let ret = func(run_ref);

            // Automatic removal once it's finished (if pool still exists)
            if let Some(inner) = weak_inner.upgrade() {
                let _ = inner.remove_by_id(id);
            }
            ret
        });

        let task = Task {
            meta: TaskMetadata {
                name: name.clone(),
                started: chrono::Utc::now(),
                id,
            },
            inner: TaskInner::Sync {
                run: Mutex::new(Some(run)),
                join_handle: Mutex::new(Some(handle)),
            },
        };

        self.inner.tasks.write().unwrap().insert(name, task);
    }

    #[cfg(feature = "async")]
    pub fn spawn_async<F>(&self, name: &str, func: F)
    where
        F: Future<Output = TaskOutput> + Send + Sync + 'static,
    {
        let _ = self.remove(&name);
        let id = self.gen_task_id();
        let weak_inner = Arc::downgrade(&self.inner);
        let name = name.to_string();

        let handle: tokio::task::JoinHandle<TaskOutput> = tokio::spawn(async move {
            let res = func.await;
            // Automatic removal once it's finished (if pool still exists)
            if let Some(inner) = weak_inner.upgrade() {
                let _ = inner.remove_by_id(id);
            }
            res
        });

        let task = Task {
            meta: TaskMetadata {
                name: name.clone(),
                started: chrono::Utc::now(),
                id,
            },
            inner: TaskInner::Async {
                join_handle: Mutex::new(Some(handle)),
            },
        };

        self.inner.tasks.write().unwrap().insert(name, task);
    }

    fn remove_by_id(&self, id: u64) -> HackshellResult<()> {
        self.inner.remove_by_id(id)
    }

    pub fn remove(&self, name: &str) -> HackshellResult<()> {
        let (_, task) = self
            .inner
            .tasks
            .write()
            .unwrap()
            .remove_entry(name)
            .ok_or(HackshellError::TaskNotFound)?;

        task.kill()?;

        Ok(())
    }

    pub fn kill_all(&self) {
        self.inner.kill_all()
    }

    pub fn join(&self, name: &str) -> HackshellResult<TaskOutput> {
        let task = self.inner.tasks.write().unwrap().remove(name);

        if let Some(task) = task {
            match task.join() {
                // If Ok() the task finished successfully. The task has already been removed.
                Ok(ret) => Ok(ret),
                Err(e) => {
                    if !matches!(e, HackshellError::JoinError(JoinError::AlreadyJoining)) {
                        let _ = self.remove_by_id(task.meta().id);
                    }

                    Err(e)
                }
            }
        } else {
            Ok(None)
        }
    }

    #[cfg(feature = "async")]
    pub async fn join_async(&self, name: &str) -> HackshellResult<TaskOutput> {
        let task = self.inner.tasks.write().unwrap().remove(name);

        if let Some(task) = task {
            // std::mem::drop(tasks);

            match task.join_async().await {
                Ok(ret) => Ok(ret),
                Err(e) => {
                    if !matches!(e, HackshellError::JoinError(JoinError::AlreadyJoining)) {
                        let _ = self.remove_by_id(task.meta().id);
                    }
                    Err(e)
                }
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_all(&self) -> Vec<TaskMetadata> {
        self.inner
            .tasks
            .read()
            .unwrap()
            .iter()
            .map(|item| item.1.meta())
            .collect::<Vec<TaskMetadata>>()
    }
}
