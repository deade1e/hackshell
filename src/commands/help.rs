use std::collections::BTreeMap;

use crate::{Command, CommandEntry, CommandResult, Hackshell};

pub struct Help {}

impl Command for Help {
    fn commands(&self) -> &'static [&'static str] {
        &["help"]
    }

    fn help(&self) -> &'static str {
        "Displays this message"
    }

    fn category(&self) -> &'static str {
        "Shell"
    }

    fn run(&self, s: &Hackshell, _: &[&str]) -> CommandResult {
        let commands = s.get_commands();
        let mut printed = vec![];
        let mut by_category: BTreeMap<&'static str, Vec<CommandEntry>> = BTreeMap::new();

        for c in commands {
            let already = printed.iter().any(|e: &CommandEntry| e.ptr_eq(&c));

            if !already {
                by_category.entry(c.category()).or_default().push(c.clone());
                printed.push(c.clone());
            }
        }

        for (category, cmds) in by_category {
            eprintln!("\n[{}]", category);
            eprintln!("{:<24} {:<24}", "Command", "Description");
            eprintln!("{:<24} {:<24}", "-------", "-----------");

            for c in cmds {
                eprintln!("{:<24} {:<24}", c.commands().join(", "), c.help());
            }
        }

        eprintln!();

        Ok(None)
    }
}
