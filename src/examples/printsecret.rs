
use hackshell::{error::MapErrToString, Command, Hackshell};

struct MyContext {
    secret: String
}

struct PrintSecret {}

#[async_trait::async_trait]
impl Command<MyContext> for PrintSecret {
    fn commands(&self) -> &'static [&'static str] {
        &["printsecret"]
    }

    fn help(&self) -> &'static str {
        "This is a non-default command installed by the Hackshell consumer. It prints a variable inside the passed context."
    }

    async fn run(
        &self,
        _s: &Hackshell<MyContext>,
        _cmd: &[String],
        ctx: &MyContext,
    ) -> Result<(), String> {
        println!("{}", ctx.secret);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let ctx = MyContext {
        secret: "It rains red in some parts of the world".to_string()
    };

    let s = Hackshell::new(ctx, "hackshell> ", None)
        .await
        .to_estring()?;

    s.add_command(PrintSecret {}).await;

    loop {
        s.run().await.to_estring()?;
    }
}
