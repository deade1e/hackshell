use crate::{Command, Hackshell};

pub struct Exit {}


#[async_trait::async_trait]
impl<C, I> Command<C, I> for Exit {
    fn commands(&self) -> &'static [&'static str] {
        &["exit"]
    }

    async fn run(&self, _s: &Hackshell<C, I>, cmd: &[String], _ctx: &C) -> Result<(), String> {
        std::process::exit(0);
    }
}
