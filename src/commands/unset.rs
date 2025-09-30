use crate::{Command, CommandResult, Hackshell};

pub struct Unset {}

impl<C: 'static> Command<C> for Unset {
    fn commands(&self) -> &'static [&'static str] {
        &["unset"]
    }

    fn help(&self) -> &'static str {
        "Unsets an environment variable"
    }

    fn run(&self, s: &Hackshell<C>, cmd: &[&str]) -> CommandResult {
        if cmd.len() != 2 {
            return Err("Syntax: unset <name>".into());
        }

        s.unset_var(&cmd[1]);

        Ok(None)
    }
}
