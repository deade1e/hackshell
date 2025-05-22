use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use tokio::{
    sync::{Mutex, RwLock},
    task::{AbortHandle, JoinHandle},
};

#[derive(Clone)]
pub struct TaskMetadata {
    pub name: String,
    pub started: chrono::DateTime<chrono::Utc>,
}

struct Task {
    meta: TaskMetadata,

    /// This is used to signal a sync thread to stop gracefully.
    /// In Rust, due to memory safety, it's not possible to stop normal threads, as they have no
    /// yelding points.
    run: Mutex<Option<Arc<AtomicBool>>>,
    wait_handle: Mutex<Option<JoinHandle<()>>>,
    abort_handle: AbortHandle,
}

#[derive(Default)]
struct InnerTaskPool {
    tasks: RwLock<HashMap<String, Arc<Task>>>,
}

#[derive(Default)]
pub struct TaskPool {
    inner: Arc<InnerTaskPool>,
}

impl Clone for TaskPool {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Task {
    pub async fn kill(&self) -> Result<(), String> {
        self.abort_handle.abort();
        if let Some(run) = self.run.lock().await.take() {
            // Signaling to a sync thread, that has no yelding points, to stop.
            run.store(false, Ordering::Relaxed);
        }
        Ok(())
    }

    pub async fn wait(&self) {
        if let Some(handle) = self.wait_handle.lock().await.take() {
            let _ = handle.await;
        }
    }
}

impl TaskPool {
    pub async fn spawn_blocking<F: Fn(Arc<AtomicBool>) + Send + 'static>(
        &self,
        name: &str,
        func: F,
    ) {
        let run = Arc::new(AtomicBool::new(true));
        let run_clone = run.clone();
        let task = tokio::task::spawn_blocking(move || {
            func(run_clone);
        });
        self.insert(name, task, Some(run)).await;
    }

    pub async fn spawn(&self, name: &str, fut: impl Future<Output = ()> + Send + 'static) {
        let task = tokio::spawn(fut);
        self.insert(name, task, None).await;
    }

    async fn insert(&self, name: &str, task: JoinHandle<()>, run: Option<Arc<AtomicBool>>) {
        let name = name.to_string();
        let self_ref = self.clone();

        // There could or could not be the task with the same name.
        // In the case it's there, we kill it and insert the new one.
        let _ = self.remove(&name).await;

        self.inner.tasks.write().await.insert(
            name.clone(),
            Arc::new(Task {
                meta: TaskMetadata {
                    name: name.clone(),
                    started: chrono::Utc::now(),
                },
                run: Mutex::new(run),
                abort_handle: task.abort_handle(),
                wait_handle: Mutex::new(Some(tokio::spawn(async move {
                    match task.await {
                        // Task finished on its own, without being aborted.
                        // Automatically removing it from the pool.
                        Ok(_) => {
                            let _ = self_ref.remove(&name).await;
                        }
                        // Task terminated somehow and produced Err, but was not cancelled.
                        Err(e) if !e.is_cancelled() => {
                            let _ = self_ref.remove(&name).await;
                        }
                        // Task was cancelled, removal will happen shortly
                        _ => {}
                    }
                }))),
            }),
        );
    }

    pub async fn remove(&self, name: &str) -> Result<(), String> {
        let task = self
            .inner
            .tasks
            .read()
            .await
            .get(name)
            .ok_or("Task not found")?
            .clone();

        task.kill().await?;

        self.inner
            .tasks
            .write()
            .await
            .remove(name)
            .ok_or("Failed to remove task from the pool")?;

        Ok(())
    }

    pub async fn wait(&self, name: &str) {
        let tasks = self.inner.tasks.read().await;
        if let Some(task) = tasks.get(name).cloned() {
            std::mem::drop(tasks);
            task.wait().await;
            // Killing and removal are automatic
        }
    }

    pub async fn get_all(&self) -> Vec<TaskMetadata> {
        self.inner
            .tasks
            .read()
            .await
            .iter()
            .map(|item| item.1.meta.clone())
            .collect::<Vec<TaskMetadata>>()
    }
}
