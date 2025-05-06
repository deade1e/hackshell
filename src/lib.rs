use readline::{Event, Readline};
use std::{collections::HashMap, path::Path, sync::Arc};
use taskpool::TaskPool;
use tokio::{
    io::{Stdin, stdin},
    task::JoinHandle,
};

mod commands;
mod error;
mod taskpool;

use commands::{exit::Exit, sleep::Sleep};
use error::MapErrToString;

// struct Task {
//     name: String,
//     started: chrono::DateTime<chrono::Utc>,
//     alive: bool,
//     terminate: watch::Sender<()>,
// }

#[async_trait::async_trait]
pub trait Command<C>: Send + Sync + 'static {
    fn commands(&self) -> &'static [&'static str];

    fn help(&self) -> &'static str;

    async fn run(&self, cmd: &[String], ctx: &C) -> Result<(), String>;
}

struct InnerHackshell<C> {
    ctx: C,
    rl: Readline<Stdin>,
    commands: HashMap<String, Arc<dyn Command<C>>>,
    pool: TaskPool,
    // tasks: RwLock<Vec<Task>>,
}

pub struct Hackshell<C> {
    inner: InnerHackshell<C>,
}

impl<C> Hackshell<C> {
    pub async fn new(ctx: C, prompt: &str, history_file: Option<&Path>) -> Self {
        let mut s = Self {
            inner: InnerHackshell {
                ctx,
                rl: Readline::new(stdin(), prompt, history_file).await,
                commands: Default::default(),
                pool: Default::default(),
            },
        };

        s.add_command(Sleep {});
        s.add_command(Exit {});

        s
    }

    pub fn add_command(&mut self, command: impl Command<C> + 'static) {
        let c = Arc::new(command);

        c.commands().iter().for_each(|cmd| {
            self.inner.commands.insert(cmd.to_string(), c.clone());
        })
    }

    pub fn get_ctx(&self) -> &C {
        &self.inner.ctx
    }

    pub async fn spawn(&self, name: &str, fut: impl Future<Output = ()> + Send + 'static) {
        self.inner.pool.spawn(name, fut).await;
    }

    pub async fn kill(&self, name: &str) -> Result<(), String> {
        self.inner.pool.kill(name).await
    }
}

impl<C: 'static> Hackshell<C> {
    pub async fn run(&self) -> Result<(), String> {
        Readline::<Stdin>::enable_raw_mode().to_estring()?;
        let line = self.inner.rl.run().await.to_estring()?;
        Readline::<Stdin>::disable_raw_mode().to_estring()?;

        match line {
            Event::Line(line) => {
                let lexer = shlex::Shlex::new(&line);
                let cmd: Vec<String> = lexer.collect();

                if cmd.is_empty() {
                    return Ok(());
                }

                if let Some(c) = self.inner.commands.get(&cmd[0]) {
                    c.run(&cmd, &self.inner.ctx)
                        .await
                        .to_estring()?;
                } else {
                    eprintln!("Command not found");
                }
            }
            Event::CTRLC => {}
            Event::EOF => {
                return Err("EOF".to_string());
            }
            Event::TAB => {}
            _ => {}
        }

        Ok(())
    }
}
