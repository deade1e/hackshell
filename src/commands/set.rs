use crate::{Command, CommandResult, Hackshell};

pub struct Set {}

impl<C: 'static> Command<C> for Set {
    fn commands(&self) -> &'static [&'static str] {
        &["set"]
    }

    fn help(&self) -> &'static str {
        "Sets an environment variable. Syntax: set <name> <value>"
    }

    fn run(&self, s: &Hackshell<C>, cmd: &[String]) -> CommandResult {
        if cmd.len() != 3 {
            return Err("Syntax: set <name> <value>".into());
        }

        s.set_var(&cmd[1], &cmd[2]);

        Ok(())
    }
}
