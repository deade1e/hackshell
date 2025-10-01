use std::sync::Arc;

use crate::{Command, CommandEntry, CommandResult, Hackshell};

pub struct Help {}

impl Command for Help {
    fn commands(&self) -> &'static [&'static str] {
        &["help"]
    }

    fn help(&self) -> &'static str {
        "Displays this message"
    }

    fn run(&mut self, s: &Hackshell, _: &[&str]) -> CommandResult {
        let commands = s.get_commands();
        let mut printed = vec![];

        eprintln!("\n{:<24} {:<24}", "Command", "Description");
        eprintln!("{:<24} {:<24}\n", "----", "----------");

        for c in commands {
            let already = printed
                .iter()
                .any(|e: &CommandEntry| Arc::ptr_eq(&e.inner, &c.inner));

            if !already {
                eprintln!("{:<24} {:<24}", c.commands().join(", "), c.help());
                printed.push(c.clone());
            }
        }

        eprintln!();

        Ok(None)
    }
}
