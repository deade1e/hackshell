use crate::{Command, CommandResult, Hackshell};

pub struct Env {}

impl Command for Env {
    fn commands(&self) -> &'static [&'static str] {
        &["env"]
    }

    fn help(&self) -> &'static str {
        "Prints all environment"
    }

    fn run(&mut self, s: &Hackshell, _: &[&str]) -> CommandResult {
        for v in s.env() {
            println!("{}={}", v.0, v.1);
        }

        Ok(None)
    }
}
