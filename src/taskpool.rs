use std::{collections::HashMap, sync::Arc};

use tokio::{
    sync::{
        RwLock,
        watch::{self, Sender},
    },
    task::JoinHandle,
};

use crate::error::MapErrToString;

pub struct Task {
    check_handle: JoinHandle<()>,
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

impl TaskPool {
    pub async fn spawn(&self, name: &str, fut: impl Future<Output = ()> + Send + 'static) {
        let name = name.to_string();
        let handle = tokio::spawn(fut);
        let (tx, mut rx) = watch::channel(());
        let self_ref = self.clone();

        self.inner.tasks.write().await.insert(
            name.clone(),
            Task {
                check_handle: tokio::spawn(async move {
                    let abrt = handle.abort_handle();

                    tokio::select! {
                        _ = handle => {},
                        _ = rx.changed() => {abrt.abort();}
                    }

                    let _ = self_ref.kill(&name).await;
                }),
                terminate: tx,
            },
        );
    }

    pub async fn kill(&self, name: &str) -> Result<(), String> {
        self.inner
            .tasks
            .read()
            .await
            .get(name)
            .ok_or("Task not found")?
            .terminate
            .send(())
            .to_estring()?;

        self.inner
            .tasks
            .write()
            .await
            .remove(name)
            .ok_or("Failed to remove task from the pool")?;

        Ok(())
    }
}
