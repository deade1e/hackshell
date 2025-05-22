use crate::{Command, Hackshell};

pub struct Help {}

impl<C: Send + Sync + 'static> Command<C> for Help {
    fn commands(&self) -> &'static [&'static str] {
        &["help"]
    }

    fn help(&self) -> &'static str {
        "Displays this message"
    }

    fn run(&self, s: &mut Hackshell<C>, _: &[String]) -> Result<(), String> {
        let commands = s.get_commands();

        eprintln!("\n{:<24} {:<24}", "Command", "Description");
        eprintln!("{:<24} {:<24}\n", "----", "----------");

        for c in commands {
            eprintln!("{:<24} {:<24}", c.commands().join(", "), c.help());
        }

        eprintln!();

        Ok(())
    }
}
