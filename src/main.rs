use hackshell::Hackshell;

struct Context {}

#[tokio::main]
async fn main() -> Result<(), String> {
    let ctx = Context {};
    let s = Hackshell::new(ctx, "> ", None).await;

    loop {
        s.run().await?;
    }
}
