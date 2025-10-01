use clap::Parser;

use crate::{Command, CommandResult, Hackshell};

#[derive(Parser, Debug)]
struct Cmd {
    /// Terminate the task
    #[clap(short = 't', long)]
    pub terminate: Option<String>,

    /// Wait the task. This command blocks the shell until the task ends.
    #[clap(short = 'w', long)]
    pub wait: Option<String>,
}

pub struct Task {}

impl Command for Task {
    fn commands(&self) -> &'static [&'static str] {
        &["task"]
    }

    fn help(&self) -> &'static str {
        "Lists and manages tasks"
    }

    fn run(&mut self, s: &Hackshell, cmd: &[&str]) -> CommandResult {
        let args = Cmd::try_parse_from(cmd)?;

        if let Some(name) = args.terminate {
            s.terminate(&name)?;
            return Ok(None);
        }

        if let Some(name) = args.wait {
            s.join(&name)?;
            return Ok(None);
        }

        let tasks = s.get_tasks();

        if tasks.is_empty() {
            eprintln!("No running tasks");
            return Ok(None);
        }

        // Print a cool table header
        eprintln!("\n{:<24} {:<24}", "Task", "Started at");
        eprintln!("{:<24} {:<24}\n", "----", "----------");

        // For each task print its name and start time
        // If there is none, just print a kind message
        for task in tasks {
            eprintln!(
                "{:<24} {:<24}",
                task.name,
                task.started.format("%Y-%m-%d %H:%M:%S")
            );
        }

        eprintln!();

        Ok(None)
    }
}
