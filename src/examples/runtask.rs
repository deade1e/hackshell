use std::time::Duration;

use hackshell::{Command, Hackshell, error::MapErrToString};
use tokio::time::sleep;

struct MyContext {}

struct RunTask {}

#[async_trait::async_trait]
impl Command<MyContext> for RunTask {
    fn commands(&self) -> &'static [&'static str] {
        &["runtask"]
    }

    fn help(&self) -> &'static str {
        "This is a non-default command installed by the Hackshell consumer. It simply spawns a task that lasts n seconds in the background."
    }

    async fn run(
        &self,
        s: &Hackshell<MyContext>,
        cmd: &[String],
        _ctx: &MyContext,
    ) -> Result<(), String> {
        if cmd.len() < 2 {
            return Err("Syntax: runtask <nsecs>".to_string());
        }

        // .to_estring() comes from the hackshell::error::MapErrToString trait
        let n = cmd[1].parse::<u64>().to_estring()?;

        s.spawn("runtask", async move {
            println!("RunTask started. Use the `task` command to see it!\r");
            sleep(Duration::from_secs(n)).await;
            println!("RunTask ended\r");
        })
        .await;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let ctx = MyContext {};

    let s = Hackshell::new(ctx, "hackshell> ", None)
        .await
        .to_estring()?;

    s.add_command(RunTask {}).await;

    loop {
        s.run().await.to_estring()?;
    }
}
