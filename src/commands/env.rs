use crate::{Command, Hackshell};

pub struct Env {}

impl<C: Send + Sync + 'static> Command<C> for Env {
    fn commands(&self) -> &'static [&'static str] {
        &["env"]
    }

    fn help(&self) -> &'static str {
        "Prints all environment"
    }

    fn run(&self, s: &mut Hackshell<C>, _: &[String]) -> Result<(), String> {
        for v in s.env() {
            println!("{}={}", v.0, v.1);
        }

        Ok(())
    }
}
