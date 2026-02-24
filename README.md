# Hackshell

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

Hackshell is a lightweight, customizable shell framework built in Rust.
It provides an interactive command-line interface that can be easily extended
with custom commands and integrated into your applications.

## Features

- **Task Management**: Background task spawning, monitoring, and killing
- **Environment Variables**: Built-in environment variable storage and manipulation
- **Rich Command Set**: Comes with essential built-in commands like `help`, `set`, `get`, `env`, etc.
- **Command History**: Persistent command history between sessions
- **Async**: Asynchronous commands and tasks via `async` feature

## Demo

![demo](assets/demo.gif)

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

You can find complete examples in the `examples` directory.
The following are quick examples.

### Basic Example

```rust
use hackshell::{Hackshell, error::HackshellError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new shell
    let shell = Hackshell::new("basic> ")?;
    shell.set_history_file("history.txt")?;

    // Enter the shell loop
    loop {
        match shell.run() {
            Ok(_) => {}
            Err(e) => {
                if matches!(e, HackshellError::Eof)
                    || matches!(e, HackshellError::Interrupted)
                    || matches!(e, HackshellError::Exit)
                {
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

You can extend Hackshell with your own commands. Commands can specify a category
which is used by the `help` command to group related commands together.
If not specified, commands default to the `"User"` category:

```rust
use hackshell::{Hackshell, Command, CommandResult};

struct MyCommand;

impl Command for MyCommand {
    fn commands(&self) -> &'static [&'static str] {
        &["mycmd"]
    }

    fn help(&self) -> &'static str {
        "mycmd - My custom command"
    }

    fn category(&self) -> &'static str {
        "Custom"  // Groups this command under [Custom] in help output
    }

    fn run(&self, _shell: &Hackshell, args: &[&str]) -> CommandResult {
        println!("My custom command was called with args: {:?}", &args[1..]);
        Ok(None)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shell = Hackshell::new("hackshell> ")?;
    shell.set_history_file("something.txt")?;

    // Add your custom command
    shell.add_command(MyCommand {});

    // Run shell loop
    // ...

    Ok(())
}
```

## Background Tasks

Hackshell allows you to spawn and manage background tasks:

```rust
use hackshell::Hackshell;
use std::sync::atomic::Ordering;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shell = Hackshell::new("> ")?;

    shell.spawn("my-task", |run| {
        while run.load(Ordering::Relaxed) {
            // do work...
        }
        None
    });

    for task in shell.get_tasks() {
        println!("Task: {}, started: {}", task.name, task.started);
    }

    shell.terminate("my-task")?;
    Ok(())
}
```

It also support asynchronous tasks!

## Forking Shells

Create a child shell that inherits the parent's environment:

```rust
use hackshell::Hackshell;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shell = Hackshell::new("> ")?;
    shell.set_var("foo", "bar");

    let child = shell.fork("subshell> ")?;
    assert_eq!(shell.get_var("foo"), child.get_var("foo"));
    Ok(())
}
```

## Installation

Add Hackshell to your `Cargo.toml`:

```toml
[dependencies]
hackshell = "0.6.1"
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
