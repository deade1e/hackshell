use crate::{Command, CommandResult, Hackshell};

const TASK_HELP: &str = "\
Usage: task [OPTIONS]

Options:
  -t, --terminate <name>  Terminate the task
  -w, --wait <name>       Wait for the task (blocks until it ends)
  --hidden                Show hidden tasks in the listing
  -h, --help              Print this help message
";

pub struct Task {}

impl Command for Task {
    fn commands(&self) -> &'static [&'static str] {
        &["task"]
    }

    fn help(&self) -> &'static str {
        "Lists and manages tasks"
    }

    fn category(&self) -> &'static str {
        "Shell"
    }

    fn run(&self, s: &Hackshell, cmd: &[&str]) -> CommandResult {
        let mut include_hidden = false;

        match cmd.get(1).map(|s| s.as_ref()) {
            Some("-h" | "--help") => {
                eprint!("{}", TASK_HELP);
                return Ok(None);
            }
            Some("-t" | "--terminate") => {
                let name = cmd.get(2).ok_or("Missing task name for --terminate")?;
                s.terminate(name)?;
                return Ok(None);
            }
            Some("-w" | "--wait") => {
                let name = cmd.get(2).ok_or("Missing task name for --wait")?;
                s.join(name)?;
                return Ok(None);
            }
            Some("--hidden") => {
                include_hidden = true;
            }
            Some(flag) if flag.starts_with('-') => {
                return Err(format!("Unknown flag: {}", flag).into());
            }
            _ => {}
        }

        let tasks = s.get_tasks_filtered(include_hidden);

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
