# Hackshell

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

Hackshell is a lightweight, customizable shell framework built in Rust. It provides an interactive command-line interface that can be easily extended with custom commands and integrated into your applications.

## Features

- **Async Command Processing**: Built on Tokio for efficient async operations
- **Task Management**: Background task spawning, monitoring, and killing
- **Environment Variables**: Built-in environment variable storage and manipulation
- **Rich Command Set**: Comes with essential built-in commands like `help`, `set`, `get`, `env`, etc.
- **Command History**: Persistent command history between sessions
- **Custom Context**: Bring your own application context for deep integration

## Built-in Commands

Hackshell comes with several built-in commands:

- `env` - List all environment variables
- `get <name>` - Get the value of an environment variable
- `set <name> <value>` - Set an environment variable
- `unset <name>` - Remove an environment variable
- `help` - Show available commands and their descriptions
- `sleep <seconds>` - Sleep for the specified duration
- `exit` - Exit the shell
- `task` - Manage background tasks

## Usage

### Basic Example

```rust
use std::path::Path;
use hackshell::Hackshell;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new shell with a custom context (in this case just ())
    let shell = Hackshell::new((), "hackshell> ", Some(Path::new("history.txt"))).await?;
    
    // Enter the shell loop
    loop {
        match shell.run().await {
            Ok(_) => {}
            Err(e) => {
                if e == "EOF" || e == "CTRLC" || e == "exit" {
                    break;
                }
                eprintln!("Error: {}", e);
            }
        }
    }
    
    Ok(())
}
```

### Adding Custom Commands

You can extend Hackshell with your own commands:

```rust
use std::path::Path;
use hackshell::{Hackshell, Command};

struct MyCommand;

#[async_trait::async_trait]
impl Command<()> for MyCommand {
    fn commands(&self) -> &'static [&'static str] {
        &["mycmd"]
    }

    fn help(&self) -> &'static str {
        "mycmd - My custom command"
    }

    async fn run(&self, shell: &Hackshell<()>, args: &[String], _ctx: &()) -> Result<(), String> {
        println!("My custom command was called with args: {:?}", &args[1..]);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shell = Hackshell::new((), "hackshell> ", Some(Path::new("history.txt"))).await?;
    
    // Add your custom command
    shell.add_command(MyCommand {}).await;
    
    // Run shell loop
    // ...
    
    Ok(())
}
```

### Advanced Context Example

```rust
struct AppState {
    config: HashMap<String, String>,
    client: DatabaseClient,
}

// Your custom commands can now access the AppState
struct ConfigCommand;

#[async_trait::async_trait]
impl Command<AppState> for ConfigCommand {
    // Implementation omitted for brevity
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = AppState {
        config: HashMap::new(),
        client: DatabaseClient::connect("localhost:5432").await?,
    };
    
    let shell = Hackshell::new(app_state, "myapp> ", Some(Path::new("history.txt"))).await?;
    // ...
}
```

## Background Tasks

Hackshell allows you to spawn and manage background tasks:

```rust
// Spawn a background task
shell.spawn("my-task", async {
    for i in 0..10 {
        println!("Background task: {}", i);
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}).await;

// List active tasks
let tasks = shell.get_tasks().await;
for task in tasks {
    println!("Task: {}, running for {}s", task.name, task.duration.as_secs());
}

// Kill a task
shell.kill("my-task").await?;
```

## Installation

Add Hackshell to your `Cargo.toml`:

```toml
[dependencies]
hackshell = "0.1.0"
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
