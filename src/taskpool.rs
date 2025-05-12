use std::{collections::HashMap, sync::Arc};

use tokio::{
    sync::{
        RwLock,
        watch::{self, Sender},
    },
    task::JoinHandle,
};

use crate::error::MapErrToString;

#[derive(Clone)]
pub struct TaskMetadata {
    pub name: String,
    pub started: chrono::DateTime<chrono::Utc>,
}

pub struct Task {
    meta: TaskMetadata,
    _check_handle: JoinHandle<()>,
    terminate: Sender<()>,
}

#[derive(Default)]
struct InnerTaskPool {
    tasks: RwLock<HashMap<String, Task>>,
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
    pub fn kill(&self) -> Result<(), String> {
        self.terminate.send(()).to_estring()
    }
}

impl TaskPool {
    pub async fn spawn(&self, name: &str, fut: impl Future<Output = ()> + Send + 'static) {
        let name = name.to_string();
        let task = tokio::spawn(fut);
        let (tx, mut rx) = watch::channel(());
        let self_ref = self.clone();

        let _ = self.kill(&name).await; // There could or could not be the task with the same name.
        // In the case it's there, we kill it and insert the new one.

        self.inner.tasks.write().await.insert(
            name.clone(),
            Task {
                meta: TaskMetadata {
                    name: name.clone(),
                    started: chrono::Utc::now(),
                },
                _check_handle: tokio::spawn(async move {
                    let abrt = task.abort_handle();

                    tokio::select! {
                        _ = task => {},
                        _ = rx.changed() => {abrt.abort();}
                    }

                    let _ = self_ref.kill(&name).await;
                }),
                terminate: tx,
            },
        );
    }

    pub async fn kill(&self, name: &str) -> Result<(), String> {
        let mut tasks = self.inner.tasks.write().await;

        tasks.get(name).ok_or("Task not found")?.kill()?;

        tasks
            .remove(name)
            .ok_or("Failed to remove task from the pool")?;

        Ok(())
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
