use std::time::Duration;

use tokio::time::sleep;

use crate::{Command, Hackshell};

pub struct Sleep {}

#[async_trait::async_trait]
impl<C, I> Command<C, I> for Sleep {
    fn commands(&self) -> &'static [&'static str] {
        &["sleep"]
    }

    fn help(&self) -> &'static str {
        "Sleeps for a specific amount of time. Syntax: sleep <seconds>"
    }

    async fn run(&self, _s: &Hackshell<C, I>, cmd: &[String], _ctx: &C) -> Result<(), String> {
        if cmd.len() == 2 {
            let duration = cmd[1].parse::<u64>().map_err(|e| e.to_string())?;
            sleep(Duration::from_secs(duration)).await;
            return Ok(());
        } else {
            return Err("Invalid number of arguments".to_string());
        }
    }
}
