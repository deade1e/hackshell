use std::{path::Path, sync::Arc};

use tokio::io::{self};

mod commands;

use commands::sleep::Sleep;

// struct Task {
//     name: String,
//     started: chrono::DateTime<chrono::Utc>,
//     alive: bool,
//     terminate: watch::Sender<()>,
// }

pub enum Event {
    Line(String),
    CTRLC,
    EOF,    // CTRL + d
    TAB,
    SUB     // CTRL + z
}

pub trait InputProvider {
    async fn read_line(&self) -> io::Result<Event>;
}

#[async_trait::async_trait]
pub trait Command<C, I>: Send + Sync + 'static {
    fn commands(&self) -> &'static [&'static str];

    async fn run(&self, s: &Hackshell<C, I>, cmd: &[&str], ctx: &C) -> Result<(), String>;
}

struct InnerHackshell<C, I> {
    ctx: C,
    ip: I,
    commands: Vec<Box<dyn Command<C, I>>>, // TODO: Transform me in a map please!
    // tasks: RwLock<Vec<Task>>,
}

pub struct Hackshell<C, I> {
    inner: Arc<InnerHackshell<C, I>>,
}

impl<C, I> Hackshell<C, I> {
    pub async fn new(ctx: C, ip: I, prompt: &str, history_file: Option<&Path>) -> Self {
        Self {
            inner: Arc::new(InnerHackshell {
                ctx,
                ip,
                commands: Default::default()
                // tasks: Default::default()
            }),
        }
    }
}

impl <C, I: InputProvider> Hackshell<C, I> {
    pub async fn run(&self) -> io::Result<()> {
        let line = self.inner.ip.read_line().await?;


        

        Ok(()) 
    }
}
