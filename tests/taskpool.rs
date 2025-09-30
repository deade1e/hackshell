use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use hackshell::taskpool::TaskPool;

#[test]
fn test_spawn_and_execute_task() {
    let pool = TaskPool::default();
    let executed = Arc::new(AtomicBool::new(false));
    let executed_clone = executed.clone();

    pool.spawn("test_task", move |run| {
        while run.load(Ordering::Relaxed) {
            executed_clone.store(true, Ordering::Relaxed);
            break;
        }
        None
    });

    thread::sleep(Duration::from_millis(50));
    assert!(executed.load(Ordering::Relaxed));
}

#[test]
fn test_task_metadata() {
    let pool = TaskPool::default();
    let task_name = "metadata_test";

    pool.spawn(task_name, |run| {
        while run.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(10));
        }
        None
    });

    let tasks = pool.get_all();
    assert_eq!(tasks.len(), 1);

    let task = &tasks[0];
    assert_eq!(task.name, task_name);
    assert!(task.started <= chrono::Utc::now());

    pool.remove(task_name).unwrap();
}

#[test]
fn test_remove_task() {
    let pool = TaskPool::default();
    let still_running = Arc::new(AtomicBool::new(true));
    let still_running_clone = still_running.clone();

    pool.spawn("removable_task", move |run| {
        while run.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(10));
        }
        still_running_clone.store(false, Ordering::Relaxed);
        None
    });

    thread::sleep(Duration::from_millis(50));
    assert!(still_running.load(Ordering::Relaxed));

    // Remove the task
    assert!(pool.remove("removable_task").is_ok());

    // Give time for the task to stop
    thread::sleep(Duration::from_millis(100));
    assert!(!still_running.load(Ordering::Relaxed));

    // Task should no longer be in the pool
    let tasks = pool.get_all();
    assert_eq!(tasks.len(), 0);
}

#[test]
fn test_remove_nonexistent_task() {
    let pool = TaskPool::default();
    assert!(pool.remove("nonexistent").is_err());
}

#[test]
fn test_join_for_task() {
    let pool = TaskPool::default();
    let completed = Arc::new(AtomicBool::new(false));
    let completed_clone = completed.clone();

    pool.spawn("join_task", move |_run| {
        thread::sleep(Duration::from_millis(100));
        completed_clone.store(true, Ordering::Relaxed);
        None
    });

    // Wait should block until task completes
    assert!(pool.join("join_task").is_ok());
    assert!(completed.load(Ordering::Relaxed));

    // Task should be automatically removed after completion
    let tasks = pool.get_all();
    assert_eq!(tasks.len(), 0);
}

#[test]
fn test_join_for_nonexistent_task() {
    let pool = TaskPool::default();
    // Waiting for a nonexistent task should return Ok (no-op)
    assert!(pool.join("nonexistent").is_ok());
}

#[test]
fn test_spawn_with_same_name_kills_previous() {
    let pool = TaskPool::default();
    let first_task_running = Arc::new(AtomicBool::new(true));
    let first_task_running_clone = first_task_running.clone();
    let second_task_started = Arc::new(AtomicBool::new(false));
    let second_task_started_clone = second_task_started.clone();

    // Spawn first task
    pool.spawn("duplicate_name", move |run| {
        while run.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(10));
        }
        first_task_running_clone.store(false, Ordering::Relaxed);
        None
    });

    thread::sleep(Duration::from_millis(50));
    assert!(first_task_running.load(Ordering::Relaxed));

    // Spawn second task with same name
    pool.spawn("duplicate_name", move |_run| {
        second_task_started_clone.store(true, Ordering::Relaxed);
        thread::sleep(Duration::from_millis(50));
        None
    });

    thread::sleep(Duration::from_millis(100));

    // First task should be killed
    assert!(!first_task_running.load(Ordering::Relaxed));
    // Second task should have started
    assert!(second_task_started.load(Ordering::Relaxed));
}

#[test]
fn test_multiple_tasks() {
    let pool = TaskPool::default();
    let counter = Arc::new(AtomicUsize::new(0));

    for i in 0..5 {
        let counter_clone = counter.clone();
        pool.spawn(&format!("task_{}", i), move |_run| {
            counter_clone.fetch_add(1, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(50));
            None
        });
    }

    thread::sleep(Duration::from_millis(100));

    // All tasks should have incremented the counter
    assert_eq!(counter.load(Ordering::Relaxed), 5);

    // All tasks should've ended
    let tasks = pool.get_all();
    assert_eq!(tasks.len(), 0);
}

#[test]
fn test_auto_removal_on_completion() {
    let pool = TaskPool::default();

    pool.spawn("auto_remove", |_run| {
        // Task completes immediately
        None
    });

    thread::sleep(Duration::from_millis(100));

    // Task should be automatically removed after completion
    let tasks = pool.get_all();
    assert_eq!(tasks.len(), 0);
}

#[test]
fn test_clone_pool() {
    let pool1 = TaskPool::default();
    let pool2 = pool1.clone();

    pool1.spawn("task_from_pool1", |_run| {
        thread::sleep(Duration::from_millis(100));
        None
    });

    // Should be able to see the task from cloned pool
    let tasks = pool2.get_all();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].name, "task_from_pool1");

    // Should be able to remove from cloned pool
    assert!(pool2.remove("task_from_pool1").is_ok());

    // Should be gone from both pools
    assert_eq!(pool1.get_all().len(), 0);
    assert_eq!(pool2.get_all().len(), 0);
}

#[cfg(feature = "async")]
mod async_tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_spawn_async_task() {
        let pool = TaskPool::default();
        let executed = Arc::new(AtomicBool::new(false));
        let executed_clone = executed.clone();

        pool.spawn_async("async_task", async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            executed_clone.store(true, Ordering::Relaxed);
            None
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(executed.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_async_task_metadata() {
        let pool = TaskPool::default();

        pool.spawn_async("async_metadata_test", async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            None
        });

        let tasks = pool.get_all();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "async_metadata_test");

        pool.remove("async_metadata_test").unwrap();
    }

    #[tokio::test]
    async fn test_kill_async_task() {
        let pool = TaskPool::default();
        let still_running = Arc::new(AtomicBool::new(true));
        let still_running_clone = still_running.clone();

        pool.spawn_async("killable_async", async move {
            tokio::time::sleep(Duration::from_secs(10)).await;
            still_running_clone.store(false, Ordering::Relaxed);
            None
        });

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(still_running.load(Ordering::Relaxed));

        // Kill the task
        assert!(pool.remove("killable_async").is_ok());

        tokio::time::sleep(Duration::from_millis(100)).await;
        // Variable should still be "running" (true) because task was aborted, not completed
        assert!(still_running.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_join_async_task() {
        let pool = TaskPool::default();
        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = completed.clone();

        pool.spawn_async("join_async", async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            completed_clone.store(true, Ordering::Relaxed);
            None
        });

        // Wait for the async task from sync context
        assert!(pool.join_async("join_async").await.is_ok());
        assert!(completed.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_async_auto_removal() {
        let pool = TaskPool::default();

        pool.spawn_async("async_auto_remove", async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            None
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Task should be automatically removed after completion
        let tasks = pool.get_all();
        assert_eq!(tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_mixed_sync_and_async_tasks() {
        let pool = TaskPool::default();
        let sync_counter = Arc::new(AtomicUsize::new(0));
        let async_counter = Arc::new(AtomicUsize::new(0));

        let sync_counter_clone = sync_counter.clone();
        pool.spawn("sync_task", move |_run| {
            sync_counter_clone.fetch_add(1, Ordering::Relaxed);
            None
        });

        let async_counter_clone = async_counter.clone();
        pool.spawn_async("async_task", async move {
            async_counter_clone.fetch_add(1, Ordering::Relaxed);
            None
        });

        assert!(pool.join("sync_task").is_ok());
        assert!(pool.join_async("async_task").await.is_ok());

        assert_eq!(sync_counter.load(Ordering::Relaxed), 1);
        assert_eq!(async_counter.load(Ordering::Relaxed), 1);

        // Both tasks should have auto-removed
        assert_eq!(pool.get_all().len(), 0);
    }
}

#[test]
fn test_concurrent_access() {
    let pool = TaskPool::default();
    let barrier = Arc::new(std::sync::Barrier::new(3));

    let pool1 = pool.clone();
    let barrier1 = barrier.clone();
    let handle1 = thread::spawn(move || {
        barrier1.wait();
        for i in 0..10 {
            pool1.spawn(&format!("thread1_task_{}", i), |_run| {
                thread::sleep(Duration::from_millis(10));
                None
            });
        }
    });

    let pool2 = pool.clone();
    let barrier2 = barrier.clone();
    let handle2 = thread::spawn(move || {
        barrier2.wait();
        for i in 0..10 {
            pool2.spawn(&format!("thread2_task_{}", i), |_run| {
                thread::sleep(Duration::from_millis(10));
                None
            });
        }
    });

    barrier.wait();

    handle1.join().unwrap();
    handle2.join().unwrap();

    thread::sleep(Duration::from_millis(100));

    // Should have no tasks
    let tasks = pool.get_all();
    assert!(tasks.len() == 0);
}
