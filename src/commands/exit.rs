use crate::{Command, Hackshell};

pub struct Exit {}

impl<C: 'static> Command<C> for Exit {
    fn commands(&self) -> &'static [&'static str] {
        &["exit"]
    }

    fn help(&self) -> &'static str {
        "Exits the program"
    }

    fn run(&self, _: &Hackshell<C>, _cmd: &[String]) -> Result<(), String> {
        Err("exit".to_string())
    }
}
