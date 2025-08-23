use crate::{Command, CommandResult, Hackshell};

pub struct Help {}

impl<C: 'static> Command<C> for Help {
    fn commands(&self) -> &'static [&'static str] {
        &["help"]
    }

    fn help(&self) -> &'static str {
        "Displays this message"
    }

    fn run(&self, s: &Hackshell<C>, _: &[String]) -> CommandResult {
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
