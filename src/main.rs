use std::sync::Arc;

use hackshell::{Command, Hackshell, error::MapErrToString};
use tokio::sync::RwLock;

struct InnerContext {
    s: RwLock<Option<Hackshell<Context>>>,
}

struct Context {
    inner: Arc<InnerContext>,
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

struct MyCommand {}

#[async_trait::async_trait]
impl Command<Context> for MyCommand {
    fn commands(&self) -> &'static [&'static str] {
        &["mycommand"]
    }

    fn help(&self) -> &'static str {
        "I help doing thingz"
    }

    async fn run(
        &self,
        _: &Hackshell<Context>,
        _cmd: &[String],
        _ctx: &Context,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let ctx = Context {
        inner: Arc::new(InnerContext {
            s: Default::default(),
        }),
    };

    let mut s = Hackshell::new(ctx.clone(), "> ", None).await.to_estring()?;

    s.add_command(MyCommand {}).await;

    {
        let mut mut_shell = ctx.inner.s.write().await;
        *mut_shell = Some(s);
    }

    loop {
        // ctx.inner.s.read().await.as_ref().unwrap().run().await?;
        ctx.inner.s.read().await.as_ref().unwrap().run().await.to_estring()?;
    }
}
