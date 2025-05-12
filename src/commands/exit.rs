use crate::{Command, Hackshell};

pub struct Exit {}

#[async_trait::async_trait]
impl<C: Send + Sync + 'static> Command<C> for Exit {
    fn commands(&self) -> &'static [&'static str] {
        &["exit"]
    }

    fn help(&self) -> &'static str {
        "Exits the program"
    }

    async fn run(&self, _: &Hackshell<C>, _cmd: &[String], _ctx: &C) -> Result<(), String> {
        Err("exit".to_string())
    }
}
