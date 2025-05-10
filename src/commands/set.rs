use crate::{Command, Hackshell};

pub struct Set {}

#[async_trait::async_trait]
impl<C: Send + Sync + 'static> Command<C> for Set {
    fn commands(&self) -> &'static [&'static str] {
        &["set"]
    }

    fn help(&self) -> &'static str {
        "Sets an environment variable. Syntax: set <name> <value>"
    }

    async fn run(&self, s: &Hackshell<C>, cmd: &[String], _ctx: &C) -> Result<(), String> {
        if cmd.len() != 3 {
            return Err("Syntax: set <name> <value>".to_string());
        }

        s.set_var(&cmd[1], &cmd[2]).await;

        Ok(())
    }
}
