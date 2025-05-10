use crate::{Command, Hackshell};

pub struct Unset {}

#[async_trait::async_trait]
impl<C: Send + Sync + 'static> Command<C> for Unset {
    fn commands(&self) -> &'static [&'static str] {
        &["unset"]
    }

    fn help(&self) -> &'static str {
        "Unsets an environment variable"
    }

    async fn run(&self, s: &Hackshell<C>, cmd: &[String], _ctx: &C) -> Result<(), String> {
        if cmd.len() != 2 {
            return Err("Syntax: unset <name>".to_string());
        }

        s.unset_var(&cmd[1]).await;

        Ok(())
    }
}
