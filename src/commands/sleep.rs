use std::{thread::sleep, time::Duration};

use crate::{Command, CommandResult, Hackshell};

pub struct Sleep {}

impl Command for Sleep {
    fn commands(&self) -> &'static [&'static str] {
        &["sleep"]
    }

    fn help(&self) -> &'static str {
        "Sleeps for a specific amount of time. Syntax: sleep <seconds>"
    }

    fn run(&self, _: &Hackshell, cmd: &[&str]) -> CommandResult {
        if cmd.len() == 2 {
            let duration = cmd[1].parse::<u64>().map_err(|e| e.to_string())?;
            sleep(Duration::from_secs(duration));
            Ok(None)
        } else {
            Err("Invalid number of arguments".into())
        }
    }
}
