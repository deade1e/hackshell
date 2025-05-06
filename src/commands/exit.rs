use crate::Command;

pub struct Exit {}

#[async_trait::async_trait]
impl<C> Command<C> for Exit {
    fn commands(&self) -> &'static [&'static str] {
        &["exit"]
    }

    fn help(&self) -> &'static str {
        "Exits the program"
    }

    async fn run(&self, _cmd: &[String], _ctx: &C) -> Result<(), String> {
        std::process::exit(0);
    }
}
