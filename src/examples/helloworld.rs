use hackshell::{error::MapErrToString, Command, Hackshell};

struct MyContext {}

struct HelloWorld {}

#[async_trait::async_trait]
impl Command<MyContext> for HelloWorld {
    fn commands(&self) -> &'static [&'static str] {
        &["helloworld"]
    }

    fn help(&self) -> &'static str {
        "This is a non-default command installed by the Hackshell consumer. It simply prints Hello, World."
    }

    async fn run(
        &self,
        _s: &Hackshell<MyContext>,
        _cmd: &[String],
        _ctx: &MyContext,
    ) -> Result<(), String> {
        println!("Hello, World!");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let ctx = MyContext {};

    let s = Hackshell::new(ctx, "hackshell> ", None)
        .await
        .to_estring()?;

    s.add_command(HelloWorld {}).await;

    loop {
        s.run().await.to_estring()?;
    }
}
