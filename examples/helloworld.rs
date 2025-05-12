use hackshell::{Command, Hackshell, error::MapErrToString};

struct HelloWorld {}

#[async_trait::async_trait]
impl Command<()> for HelloWorld {
    fn commands(&self) -> &'static [&'static str] {
        &["helloworld"]
    }

    fn help(&self) -> &'static str {
        "This is a non-default command installed by the Hackshell consumer. It simply prints Hello, World."
    }

    async fn run(&self, _s: &Hackshell<()>, _cmd: &[String], _ctx: &()) -> Result<(), String> {
        println!("Hello, World!");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let s = Hackshell::new((), "hackshell> ", None).await.to_estring()?;

    s.add_command(HelloWorld {}).await;

    loop {
        s.run().await.to_estring()?;
    }
}
