use crate::Command;

pub struct Help {}

#[async_trait::async_trait]
impl<C> Command<C> for Help {
    fn commands(&self) -> &'static [&'static str] {
        &["help"]
    }

    fn help(&self) -> &'static str {
        "Displays this message"
    } 

    async fn run(&self, _: &[String], _: &C) -> Result<(), String> {
        Ok(())
    }
}
