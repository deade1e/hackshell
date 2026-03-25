#![cfg(feature = "async")]

use hackshell::{
    AsyncCommand, Command, CommandResult, Hackshell, TaskOptions, async_trait,
    error::HackshellError,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

// A simple sync command for testing
struct SyncCounter {
    counter: Arc<AtomicUsize>,
}

impl Command for SyncCounter {
    fn commands(&self) -> &'static [&'static str] {
        &["sync-count"]
    }

    fn help(&self) -> &'static str {
        "Increments a counter synchronously"
    }

    fn run(&self, _shell: &Hackshell, _cmd: &[&str]) -> CommandResult {
        self.counter.fetch_add(1, Ordering::Relaxed);
        Ok(Some("counted".to_string()))
    }
}

// A simple async command for testing
struct AsyncCounter {
    counter: Arc<AtomicUsize>,
}

#[async_trait]
impl AsyncCommand for AsyncCounter {
    fn commands(&self) -> &'static [&'static str] {
        &["async-count"]
    }

    fn help(&self) -> &'static str {
        "Increments a counter asynchronously"
    }

    async fn run(&self, _shell: &Hackshell, _cmd: &[&str]) -> CommandResult {
        tokio::time::sleep(Duration::from_millis(10)).await;
        self.counter.fetch_add(1, Ordering::Relaxed);
        Ok(Some("async counted".to_string()))
    }
}

// Async command that does actual async work
struct AsyncDelayCommand {
    completed: Arc<AtomicBool>,
}

#[async_trait]
impl AsyncCommand for AsyncDelayCommand {
    fn commands(&self) -> &'static [&'static str] {
        &["async-delay"]
    }

    fn help(&self) -> &'static str {
        "Delays asynchronously"
    }

    async fn run(&self, _shell: &Hackshell, _cmd: &[&str]) -> CommandResult {
        tokio::time::sleep(Duration::from_millis(50)).await;
        self.completed.store(true, Ordering::Relaxed);
        Ok(Some("delayed".to_string()))
    }
}

#[tokio::test]
async fn test_feed_slice_async_with_sync_command() {
    let shell = Hackshell::new("> ").unwrap();
    let counter = Arc::new(AtomicUsize::new(0));

    shell.add_command(SyncCounter {
        counter: counter.clone(),
    });

    let result = shell.feed_slice_async(&["sync-count"]).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some("counted".to_string()));
    assert_eq!(counter.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn test_feed_slice_async_with_async_command() {
    let shell = Hackshell::new("> ").unwrap();
    let counter = Arc::new(AtomicUsize::new(0));

    shell.add_async_command(AsyncCounter {
        counter: counter.clone(),
    });

    let result = shell.feed_slice_async(&["async-count"]).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some("async counted".to_string()));
    assert_eq!(counter.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn test_feed_line_async_parses_and_executes() {
    let shell = Hackshell::new("> ").unwrap();
    let counter = Arc::new(AtomicUsize::new(0));

    shell.add_async_command(AsyncCounter {
        counter: counter.clone(),
    });

    let result = shell.feed_line_async("async-count").await;
    assert!(result.is_ok());
    assert_eq!(counter.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn test_feed_slice_async_command_not_found() {
    let shell = Hackshell::new("> ").unwrap();

    let result = shell.feed_slice_async(&["nonexistent"]).await;
    assert!(matches!(result, Err(HackshellError::CommandNotFound)));
}

#[tokio::test]
async fn test_feed_slice_async_empty_command() {
    let shell = Hackshell::new("> ").unwrap();

    let result = shell.feed_slice_async(&[]).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
}

#[tokio::test]
async fn test_sync_feed_slice_accepts_sync_command() {
    let shell = Hackshell::new("> ").unwrap();
    let counter = Arc::new(AtomicUsize::new(0));

    shell.add_command(SyncCounter {
        counter: counter.clone(),
    });

    // Sync feed_slice should work with sync commands
    let result = shell.feed_slice(&["sync-count"]);
    assert!(result.is_ok());
    assert_eq!(counter.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn test_async_command_actually_awaits() {
    let shell = Hackshell::new("> ").unwrap();
    let completed = Arc::new(AtomicBool::new(false));

    shell.add_async_command(AsyncDelayCommand {
        completed: completed.clone(),
    });

    // Before execution
    assert!(!completed.load(Ordering::Relaxed));

    // Execute async command
    let result = shell.feed_slice_async(&["async-delay"]).await;
    assert!(result.is_ok());

    // After execution - should have awaited and completed
    assert!(completed.load(Ordering::Relaxed));
}

#[tokio::test]
async fn test_mixed_sync_and_async_commands() {
    let shell = Hackshell::new("> ").unwrap();
    let sync_counter = Arc::new(AtomicUsize::new(0));
    let async_counter = Arc::new(AtomicUsize::new(0));

    shell.add_command(SyncCounter {
        counter: sync_counter.clone(),
    });
    shell.add_async_command(AsyncCounter {
        counter: async_counter.clone(),
    });

    // Execute both types
    shell.feed_slice_async(&["sync-count"]).await.unwrap();
    shell.feed_slice_async(&["async-count"]).await.unwrap();
    shell.feed_slice_async(&["sync-count"]).await.unwrap();
    shell.feed_slice_async(&["async-count"]).await.unwrap();

    assert_eq!(sync_counter.load(Ordering::Relaxed), 2);
    assert_eq!(async_counter.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn test_async_command_can_spawn_tasks() {
    let shell = Hackshell::new("> ").unwrap();
    let task_started = Arc::new(AtomicBool::new(false));
    let task_started_clone = task_started.clone();

    struct SpawnerCommand {
        flag: Arc<AtomicBool>,
    }

    #[async_trait]
    impl AsyncCommand for SpawnerCommand {
        fn commands(&self) -> &'static [&'static str] {
            &["spawner"]
        }

        fn help(&self) -> &'static str {
            "Spawns a task"
        }

        async fn run(&self, shell: &Hackshell, _cmd: &[&str]) -> CommandResult {
            let flag = self.flag.clone();
            shell.spawn_async("spawned-task", TaskOptions::default(), async move {
                flag.store(true, Ordering::Relaxed);
                None
            });
            Ok(None)
        }
    }

    shell.add_async_command(SpawnerCommand {
        flag: task_started_clone,
    });

    shell.feed_slice_async(&["spawner"]).await.unwrap();

    // Give the spawned task time to run
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(task_started.load(Ordering::Relaxed));
}
