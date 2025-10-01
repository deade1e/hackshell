// examples/async.rs
//
// This example demonstrates how to use async tasks with hackshell
// Run with: cargo run --example async --features async

use hackshell::{Command, CommandResult, Hackshell, error::HackshellError};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

// Custom context to hold application state
struct AppContext {
    messages: Arc<Mutex<Vec<String>>>,
}

impl AppContext {
    fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

// Command to spawn an async task using the TaskPool
struct AsyncTaskCommand {
    ctx: Arc<AppContext>,
}

impl Command for AsyncTaskCommand {
    fn commands(&self) -> &'static [&'static str] {
        &["async-task", "at"]
    }

    fn help(&self) -> &'static str {
        "async-task <name> [count] - Spawn an async task using task management"
    }

    fn run(&self, shell: &Hackshell, args: &[&str]) -> CommandResult {
        if args.len() < 2 {
            println!("Usage: async-task <name> [count]");
            return Ok(None);
        }

        let task_name = args[1].to_string();
        let count = args
            .get(2)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(10);

        let messages = self.ctx.messages.clone();
        let task_name_clone = task_name.clone();

        // Using the TaskPool's spawn_async method
        shell.spawn_async(&task_name, async move {
            println!(
                "Async task '{}' started (counting to {})",
                task_name_clone, count
            );

            for i in 1..=count {
                sleep(Duration::from_millis(500)).await;

                let mut msgs = messages.lock().unwrap();
                msgs.push(format!("Task '{}': {}/{}", task_name_clone, i, count));

                if i % 5 == 0 {
                    println!("Task '{}': reached {}/{}", task_name_clone, i, count);
                }
            }

            println!("Task '{}' finished counting to {}!", task_name_clone, count);
            None
        });

        println!("Spawned async task: '{}'", task_name);

        Ok(None)
    }
}

// Command to check async task progress
struct CheckProgressCommand {
    ctx: Arc<AppContext>,
}

impl Command for CheckProgressCommand {
    fn commands(&self) -> &'static [&'static str] {
        &["progress", "p"]
    }

    fn help(&self) -> &'static str {
        "progress - Check progress of async tasks"
    }

    fn run(&self, shell: &Hackshell, _args: &[&str]) -> CommandResult {
        let messages = self.ctx.messages.lock().unwrap();

        println!("Current Status:");
        println!("   Active tasks: {}", shell.get_tasks().len());

        if !messages.is_empty() {
            println!("\nRecent messages:");
            for (i, msg) in messages.iter().rev().take(5).enumerate() {
                println!("   {}. {}", i + 1, msg);
            }
        }

        Ok(None)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hackshell Async Tasks Example");
    println!("Type 'help' to see available commands\n");

    let context = Arc::new(AppContext::new());
    let shell = Hackshell::new("async> ")?;

    // Set up history
    shell.set_history_file("history.txt")?;

    // Add our custom async commands
    shell.add_command(AsyncTaskCommand {
        ctx: context.clone(),
    });

    shell.add_command(CheckProgressCommand {
        ctx: context.clone(),
    });

    let rt = tokio::runtime::Runtime::new().unwrap();
    let handle = rt.spawn_blocking(move || {
        // Main shell loop
        loop {
            match shell.run() {
                Ok(_) => {}
                Err(e) => {
                    if matches!(e, HackshellError::Eof)
                        || matches!(e, HackshellError::Interrupted)
                        || matches!(e, HackshellError::Exit)
                    {
                        println!("\nGoodbye!");
                        break;
                    }

                    println!("Error: {}", e);
                }
            }
        }
    });

    let _ = rt.block_on(async { handle.await });

    Ok(())
}
