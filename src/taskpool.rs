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

struct Task {
    meta: TaskMetadata,

    /// This is used to signal a sync thread to stop gracefully.
    /// In Rust, due to memory safety, it's not possible to stop normal threads, as they have no
    /// yelding points.
    run: Mutex<Option<Arc<AtomicBool>>>,
    wait_handle: Mutex<Option<JoinHandle<()>>>,
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
    pub fn kill(&self) -> Result<(), String> {
        if let Some(run) = self.run.lock().unwrap().take() {
            // Signaling to a sync thread, that has no yelding points, to stop.
            run.store(false, Ordering::Relaxed);
        }
        Ok(())
    }

    pub fn wait(&self) {
        if let Some(handle) = self.wait_handle.lock().unwrap().take() {
            let _ = handle.join();
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

        let task = std::thread::spawn(move || {
            func(run_ref);

            // Automatic removal once it's finished
            let _ = self_ref.remove(&name_ref);
        });


        self.inner.tasks.write().unwrap().insert(
            name.clone(),
            Arc::new(Task {
                meta: TaskMetadata {
                    name: name.clone(),
                    started: chrono::Utc::now(),
                },
                run: Mutex::new(Some(run)),
                wait_handle: Mutex::new(Some(task)),
            }),
        );
    }

    pub fn remove(&self, name: &str) -> Result<(), String> {
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

    pub fn wait(&self, name: &str) {
        let tasks = self.inner.tasks.read().unwrap();
        if let Some(task) = tasks.get(name).cloned() {
            std::mem::drop(tasks);
            task.wait();
            // Killing and removal are automatic
        }
    }

    pub fn get_all(&self) -> Vec<TaskMetadata> {
        self.inner
            .tasks
            .read()
            .unwrap()
            .iter()
            .map(|item| item.1.meta.clone())
            .collect::<Vec<TaskMetadata>>()
    }
}
