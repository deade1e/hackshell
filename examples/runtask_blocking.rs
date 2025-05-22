use std::{thread::sleep, time::Duration};

use hackshell::{Command, Hackshell, error::MapErrToString};

struct RunTaskBlocking {}

#[async_trait::async_trait]
impl Command<()> for RunTaskBlocking {
    fn commands(&self) -> &'static [&'static str] {
        &["runtaskblk"]
    }

    fn help(&self) -> &'static str {
        "This is a non-default command installed by the Hackshell consumer. It simply spawns a sync task that lasts n seconds in the background."
    }

    async fn run(&self, s: &Hackshell<()>, cmd: &[String], _ctx: &()) -> Result<(), String> {
        if cmd.len() < 2 {
            return Err("Syntax: runtaskblk <nsecs>".to_string());
        }

        // .to_estring() comes from the hackshell::error::MapErrToString trait
        let n = cmd[1].parse::<u64>().to_estring()?;

        s.spawn_blocking("runtaskblk", move |run| {
            println!("RunTaskBlocking started. Use the `task` command to see it!. This can only be terminated by issuing `task -t runtaskblk`.\r");

            while run.load(std::sync::atomic::Ordering::Relaxed) {
                sleep(Duration::from_secs(n));
                println!("RunTaskBlocking is running!\r");
            }
        })
        .await;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let s = Hackshell::new((), "hackshell> ", None).await.to_estring()?;

    s.add_command(RunTaskBlocking {}).await;

    loop {
        s.run().await.to_estring()?;
    }
}
