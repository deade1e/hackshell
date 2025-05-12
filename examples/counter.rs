use hackshell::{Command, Hackshell, error::MapErrToString};
use tokio::sync::RwLock;

struct Counter {
    counter: RwLock<u64>,
}

#[async_trait::async_trait]
impl Command<()> for Counter {
    fn commands(&self) -> &'static [&'static str] {
        &["counter"]
    }

    fn help(&self) -> &'static str {
        "This is a non-default command installed by the Hackshell consumer. It simply increments an internal counter"
    }

    async fn run(&self, _s: &Hackshell<()>, _cmd: &[String], _ctx: &()) -> Result<(), String> {
        let mut counter = self.counter.write().await;

        *counter += 1;

        println!("The counter is now: {}\r", counter);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let s = Hackshell::new((), "hackshell> ", None).await.to_estring()?;

    s.add_command(Counter {
        counter: RwLock::new(0),
    })
    .await;

    loop {
        s.run().await.to_estring()?;
    }
}
