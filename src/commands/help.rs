use crate::{Command, Hackshell};

pub struct Help {}

#[async_trait::async_trait]
impl<C: Send + Sync + 'static> Command<C> for Help {
    fn commands(&self) -> &'static [&'static str] {
        &["help"]
    }

    fn help(&self) -> &'static str {
        "Displays this message"
    }

    async fn run(&self, s: &Hackshell<C>, _: &[String], _: &C) -> Result<(), String> {
        let commands = s.get_commands().await;

        eprintln!("\n{:<24} {:<24}", "Command", "Description");
        eprintln!("{:<24} {:<24}\n", "----", "----------");

        for c in commands {
            eprintln!("{:<24} {:<24}", c.commands().join(", "), c.help());
        }

        eprint!("\n");

        Ok(())
    }
}
