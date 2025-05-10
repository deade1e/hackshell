use crate::{Command, Hackshell};

pub struct Env {}

#[async_trait::async_trait]
impl<C: Send + Sync + 'static> Command<C> for Env {
    fn commands(&self) -> &'static [&'static str] {
        &["env"]
    }

    fn help(&self) -> &'static str {
        "Prints all environment"
    }

    async fn run(&self, s: &Hackshell<C>, cmd: &[String], _ctx: &C) -> Result<(), String> {
        for v in s.env().await {
            println!("{}={}", v.0, v.1);
        }

        Ok(())
    }
}
