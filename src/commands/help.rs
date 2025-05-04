use crate::{Command, Hackshell};

pub struct Help {}

#[async_trait::async_trait]
impl<C, I> Command<C, I> for Help {
    fn commands(&self) -> &'static [&'static str] {
        &["help"]
    }

    fn help(&self) -> &'static str {
        "Displays this message"
    } 

    async fn run(&self, s: &Hackshell<C, I>, _: &[String], _: &C) -> Result<(), String> {
        Ok(())
    }
}
