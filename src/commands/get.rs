use crate::{Command, CommandResult, Hackshell};

pub struct Get {}

impl<C: 'static> Command<C> for Get {
    fn commands(&self) -> &'static [&'static str] {
        &["get"]
    }

    fn help(&self) -> &'static str {
        "Prints an environment variable"
    }

    fn run(&self, s: &Hackshell<C>, cmd: &[&str]) -> CommandResult {
        if cmd.len() != 2 {
            return Err("Syntax: get <name>".into());
        }

        println!("{}", s.get_var(&cmd[1]).ok_or("Variable not found")?);

        Ok(None)
    }
}
