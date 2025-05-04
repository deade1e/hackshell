use hackshell::{Event, Hackshell, InputProvider};
use readline::Readline;
use tokio::io::{self, AsyncRead, Stdin, stdin};

struct Context {}
struct MyInputProvider<R> {
    rl: Readline<R>,
}

impl MyInputProvider<Stdin> {
    pub async fn new() -> Self {
        Self {
            rl: Readline::new(stdin(), "> ", None).await,
        }
    }
}

impl<R: AsyncRead + Unpin> InputProvider for MyInputProvider<R> {
    async fn read_line(&self) -> io::Result<Event> {
        Readline::<R>::enable_raw_mode()?;
        let r = Ok(Event::Line(self.rl.run().await?));
        Readline::<R>::disable_raw_mode()?;
        r
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let ctx = Context {};
    let myip = MyInputProvider::new().await;
    let s = Hackshell::new(ctx, myip, "> ", None).await;

    loop {
        s.run().await?;
    }
}
