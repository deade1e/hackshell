use crate::{Command, Hackshell};

#[derive(Clone)]
pub struct Unset {}

impl<C: Send + Sync + 'static> Command<C> for Unset {
    fn commands(&self) -> &'static [&'static str] {
        &["unset"]
    }

    fn help(&self) -> &'static str {
        "Unsets an environment variable"
    }

    fn run(&self, s: &mut Hackshell<C>, cmd: &[String]) -> Result<(), String> {
        if cmd.len() != 2 {
            return Err("Syntax: unset <name>".to_string());
        }

        s.unset_var(&cmd[1]);

        Ok(())
    }
}
