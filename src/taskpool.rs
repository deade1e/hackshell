use crate::error::{HackshellError, HackshellResult};
#[cfg(feature = "async")]
use std::pin::Pin;
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

struct SyncTask {
    meta: TaskMetadata,

    /// This is used to signal a sync thread to stop gracefully.
    /// In Rust, due to memory safety, it's not possible to stop normal threads, as they have no
    /// yelding points.
    run: Mutex<Option<Arc<AtomicBool>>>,
    wait_handle: Mutex<Option<JoinHandle<TaskOutput>>>,
}

#[cfg(feature = "async")]
struct AsyncTask {
    meta: TaskMetadata,
    wait_handle: Mutex<Option<tokio::task::JoinHandle<TaskOutput>>>,
}

trait Task {
    fn meta(&self) -> TaskMetadata;
    fn kill(&self) -> HackshellResult<()>;
    fn wait(&self) -> HackshellResult<TaskOutput>;
    #[cfg(feature = "async")]
    fn wait_async(&self) -> Pin<Box<dyn Future<Output = HackshellResult<TaskOutput>>>>;
}

#[derive(Default)]
struct InnerTaskPool {
    task_id: Arc<AtomicU64>,
    tasks: RwLock<HashMap<String, Arc<dyn Task + Send + Sync>>>,
}

#[derive(Default, Clone)]
pub struct TaskPool {
    inner: Arc<InnerTaskPool>,
}

impl Task for SyncTask {
    fn meta(&self) -> TaskMetadata {
        self.meta.clone()
    }

    fn kill(&self) -> HackshellResult<()> {
        if let Some(run) = self.run.lock().unwrap().take() {
            // Signaling to a sync thread, that has no yelding points, to stop.
            run.store(false, Ordering::Relaxed);
        }
        Ok(())
    }

    fn wait(&self) -> HackshellResult<TaskOutput> {
        let wh = self
            .wait_handle
            .lock()
            .unwrap()
            .take()
            .ok_or("Can't take wait handle")?;

        wh.join().map_err(|e| {
            HackshellError::JoinError(crate::error::JoinError::Sync(Box::new(Mutex::new(e))))
        })
    }

    #[cfg(feature = "async")]
    fn wait_async(&self) -> Pin<Box<dyn Future<Output = HackshellResult<TaskOutput>>>> {
        Box::pin(async {
            Err(HackshellError::JoinError(
                crate::error::JoinError::CannotWaitAsync,
            ))
        })
    }
}

#[cfg(feature = "async")]
impl Task for AsyncTask {
    fn meta(&self) -> TaskMetadata {
        self.meta.clone()
    }

    fn kill(&self) -> HackshellResult<()> {
        if let Some(handle) = self.wait_handle.lock().unwrap().take() {
            handle.abort();
        }

        Ok(())
    }

    fn wait(&self) -> HackshellResult<TaskOutput> {
        let wh = self
            .wait_handle
            .lock()
            .unwrap()
            .take()
            .ok_or("Can't take wait handle")?;

        let handle = tokio::runtime::Handle::try_current().map_err(|e| e.to_string())?;

        handle.block_on(async move {
            wh.await
                .map_err(|e| HackshellError::JoinError(crate::error::JoinError::Async(e)))
        })
    }

    #[cfg(feature = "async")]
    fn wait_async(&self) -> Pin<Box<dyn Future<Output = HackshellResult<TaskOutput>>>> {
        let wh = self
            .wait_handle
            .lock()
            .unwrap()
            .take()
            .ok_or("Can't take wait handle");

        match wh {
            Ok(wh) => Box::pin(async move {
                wh.await
                    .map_err(|e| HackshellError::JoinError(crate::error::JoinError::Async(e)))
            }),

            Err(e) => Box::pin(async move { Err(e.into()) }),
        }
    }
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
        let self_ref = self.clone();
        let name = name.to_string();

        // There could or could not be the task with the same name.
        // In the case it's there, we kill it and insert the new one.
        let _ = self.remove(&name);
        let id = self.gen_task_id();

        let handle = std::thread::spawn(move || {
            let ret = func(run_ref);

            // Automatic removal once it's finished
            let _ = self_ref.remove_by_id(id);
            ret
        });

        let task = SyncTask {
            meta: TaskMetadata {
                name: name.clone(),
                started: chrono::Utc::now(),
                id: id,
            },
            run: Mutex::new(Some(run)),
            wait_handle: Mutex::new(Some(handle)),
        };

        let task = Arc::new(task);
        self.inner.tasks.write().unwrap().insert(name, task);
    }

    #[cfg(feature = "async")]
    pub fn spawn_async<F>(&self, name: &str, func: F)
    where
        F: Future<Output = TaskOutput> + Send + Sync + 'static,
    {
        let _ = self.remove(&name);
        let id = self.gen_task_id();
        let self_ref = self.clone();
        let name = name.to_string();

        let handle: tokio::task::JoinHandle<Option<Box<dyn std::any::Any + Send + Sync>>> =
            tokio::spawn(async move {
                let res = func.await;
                let _ = self_ref.remove_by_id(id);
                res
            });

        let task = AsyncTask {
            meta: TaskMetadata {
                name: name.clone(),
                started: chrono::Utc::now(),
                id,
            },
            wait_handle: Mutex::new(Some(handle)),
        };

        let task = Arc::new(task);

        self.inner.tasks.write().unwrap().insert(name, task);
    }

    fn remove_by_id(&self, id: u64) -> HackshellResult<()> {
        let mut tasks = self.inner.tasks.write().unwrap();

        let key = tasks
            .iter()
            .find(|(_, v)| v.meta().id == id)
            .map(|(k, _)| k.clone())
            .ok_or("Cannot find the task")?;

        let (_, task) = tasks.remove_entry(&key).unwrap();

        task.kill()?;

        Ok(())
    }

    pub fn remove(&self, name: &str) -> HackshellResult<()> {
        let (_, task) = self
            .inner
            .tasks
            .write()
            .unwrap()
            .remove_entry(name)
            .ok_or("Task not found")?;

        task.kill()?;

        Ok(())
    }

    pub fn wait(&self, name: &str) -> HackshellResult<TaskOutput> {
        let tasks = self.inner.tasks.read().unwrap();
        if let Some(task) = tasks.get(name).cloned() {
            std::mem::drop(tasks);

            match task.wait() {
                Ok(ret) => Ok(ret),
                Err(e) => {
                    let _ = self.remove_by_id(task.meta().id);
                    Err(e)
                }
            }
        } else {
            Ok(None)
        }
    }

    #[cfg(feature = "async")]
    pub async fn wait_async(&self, name: &str) -> HackshellResult<TaskOutput> {
        let tasks = self.inner.tasks.read().unwrap();

        if let Some(task) = tasks.get(name).cloned() {
            std::mem::drop(tasks);

            match task.wait_async().await {
                Ok(ret) => Ok(ret),
                Err(e) => {
                    let _ = self.remove_by_id(task.meta().id);
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
