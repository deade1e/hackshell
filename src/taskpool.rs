use crate::error::{HackshellError, Result};
#[cfg(feature = "async")]
use std::pin::Pin;
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
};

#[derive(Clone)]
pub struct TaskMetadata {
    pub name: String,
    pub started: chrono::DateTime<chrono::Utc>,
}

struct SyncTask {
    meta: TaskMetadata,

    /// This is used to signal a sync thread to stop gracefully.
    /// In Rust, due to memory safety, it's not possible to stop normal threads, as they have no
    /// yelding points.
    run: Mutex<Option<Arc<AtomicBool>>>,
    wait_handle: Mutex<Option<JoinHandle<()>>>,
}

#[cfg(feature = "async")]
struct AsyncTask {
    meta: TaskMetadata,
    wait_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

trait Task {
    fn meta(&self) -> TaskMetadata;
    fn kill(&self) -> Result<()>;
    fn wait(&self) -> Result<()>;
    #[cfg(feature = "async")]
    fn wait_async(&self) -> Pin<Box<dyn Future<Output = Result<()>>>>;
}

#[derive(Default)]
struct InnerTaskPool {
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

    fn kill(&self) -> Result<()> {
        if let Some(run) = self.run.lock().unwrap().take() {
            // Signaling to a sync thread, that has no yelding points, to stop.
            run.store(false, Ordering::Relaxed);
        }
        Ok(())
    }

    fn wait(&self) -> Result<()> {
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
    fn wait_async(&self) -> Pin<Box<dyn Future<Output = Result<()>>>> {
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

    fn kill(&self) -> Result<()> {
        if let Some(handle) = self.wait_handle.lock().unwrap().take() {
            handle.abort();
        }

        Ok(())
    }

    fn wait(&self) -> Result<()> {
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
    fn wait_async(&self) -> Pin<Box<dyn Future<Output = Result<()>>>> {
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
    pub fn spawn<F: FnOnce(Arc<AtomicBool>) + Send + 'static>(&self, name: &str, func: F) {
        let run = Arc::new(AtomicBool::new(true));
        let run_ref = run.clone();
        let self_ref = self.clone();
        let name = name.to_string();
        let name_ref = name.clone();

        // There could or could not be the task with the same name.
        // In the case it's there, we kill it and insert the new one.
        let _ = self.remove(&name);

        let handle = std::thread::spawn(move || {
            func(run_ref);

            // Automatic removal once it's finished
            let _ = self_ref.remove(&name_ref);
        });

        let task = SyncTask {
            meta: TaskMetadata {
                name: name.clone(),
                started: chrono::Utc::now(),
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
        F: Future + Send + Sync + 'static,
        F::Output: Send + Sync,
    {
        let _ = self.remove(&name);
        let self_ref = self.clone();
        let name = name.to_string();
        let name_ref = name.clone();

        let handle = tokio::spawn(async move {
            func.await;
            let _ = self_ref.remove(&name_ref);
        });

        let task = AsyncTask {
            meta: TaskMetadata {
                name: name.clone(),
                started: chrono::Utc::now(),
            },
            wait_handle: Mutex::new(Some(handle)),
        };

        let task = Arc::new(task);

        self.inner.tasks.write().unwrap().insert(name, task);
    }

    pub fn remove(&self, name: &str) -> Result<()> {
        let task = self
            .inner
            .tasks
            .read()
            .unwrap()
            .get(name)
            .ok_or("Task not found")?
            .clone();

        task.kill()?;

        self.inner
            .tasks
            .write()
            .unwrap()
            .remove(name)
            .ok_or("Failed to remove task from the pool")?;

        Ok(())
    }

    pub fn wait(&self, name: &str) -> Result<()> {
        let tasks = self.inner.tasks.read().unwrap();
        if let Some(task) = tasks.get(name).cloned() {
            std::mem::drop(tasks);

            match task.wait() {
                Ok(()) => Ok(()),
                Err(e) => {
                    let _ = self.remove(&task.meta().name);
                    Err(e)
                }
            }
        } else {
            Ok(())
        }
    }

    #[cfg(feature = "async")]
    pub async fn wait_async(&self, name: &str) -> Result<()> {
        let tasks = self.inner.tasks.read().unwrap();

        if let Some(task) = tasks.get(name).cloned() {
            std::mem::drop(tasks);

            match task.wait_async().await {
                Ok(()) => Ok(()),
                Err(e) => {
                    let _ = self.remove(&task.meta().name);
                    Err(e)
                }
            }
        } else {
            Ok(())
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
