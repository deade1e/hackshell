use crate::{Command, Hackshell};

pub struct Get {}

#[async_trait::async_trait]
impl<C: Send + Sync + 'static> Command<C> for Get {
    fn commands(&self) -> &'static [&'static str] {
        &["get"]
    }

    fn help(&self) -> &'static str {
        "Prints an environment variable"
    }

    async fn run(&self, s: &Hackshell<C>, cmd: &[String], _ctx: &C) -> Result<(), String> {
        if cmd.len() != 2 {
            return Err("Syntax: get <name>".to_string());
        }

        println!("{}", s.get_var(&cmd[1]).await.ok_or("Variable not found")?);

        Ok(())
    }
}
