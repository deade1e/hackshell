use std::{thread::sleep, time::Duration};

use crate::{Command, Hackshell};

pub struct Sleep {}

impl<C> Command<C> for Sleep {
    fn commands(&self) -> &'static [&'static str] {
        &["sleep"]
    }

    fn help(&self) -> &'static str {
        "Sleeps for a specific amount of time. Syntax: sleep <seconds>"
    }

    fn run(&self, _: &Hackshell<C>, cmd: &[String]) -> Result<(), String> {
        if cmd.len() == 2 {
            let duration = cmd[1].parse::<u64>().map_err(|e| e.to_string())?;
            sleep(Duration::from_secs(duration));
            Ok(())
        } else {
            Err("Invalid number of arguments".to_string())
        }
    }
}
