use crate::{Command, CommandResult, Hackshell, error::HackshellError};

pub struct Exit {}

impl Command for Exit {
    fn commands(&self) -> &'static [&'static str] {
        &["exit"]
    }

    fn help(&self) -> &'static str {
        "Exits the program"
    }

    fn run(&mut self, _: &Hackshell, _cmd: &[&str]) -> CommandResult {
        Err(HackshellError::Exit.into())
    }
}
