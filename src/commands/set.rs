use crate::{Command, CommandResult, Hackshell};

pub struct Set {}

impl Command for Set {
    fn commands(&self) -> &'static [&'static str] {
        &["set"]
    }

    fn help(&self) -> &'static str {
        "Sets an environment variable. Syntax: set <name> <value>"
    }

    fn run(&self, s: &Hackshell, cmd: &[&str]) -> CommandResult {
        if cmd.len() != 3 {
            return Err("Syntax: set <name> <value>".into());
        }

        s.set_var(&cmd[1], &cmd[2]);

        Ok(None)
    }
}
