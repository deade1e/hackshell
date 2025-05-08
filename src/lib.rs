use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::{
    io::{self, Stdin, stdin},
    sync::{Mutex, RwLock},
};

mod commands;
pub mod error;
mod readline;
mod taskpool;

use crate::readline::{Event, Readline};
use commands::{exit::Exit, sleep::Sleep};
use error::MapErrToString;
use taskpool::TaskPool;

#[async_trait::async_trait]
pub trait Command<C>: Send + Sync + 'static {
    fn commands(&self) -> &'static [&'static str];

    fn help(&self) -> &'static str;

    async fn run(&self, s: &Hackshell<C>, cmd: &[String], ctx: &C) -> Result<(), String>;
}

struct InnerHackshell<C> {
    ctx: C,
    commands: RwLock<HashMap<String, Arc<dyn Command<C>>>>,
    prompt: String,
    rl: Mutex<Readline<Stdin>>,
    pool: TaskPool,
}

pub struct Hackshell<C> {
    inner: Arc<InnerHackshell<C>>,
}

impl<C> Clone for Hackshell<C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<C> Hackshell<C> {
    pub async fn new(ctx: C, prompt: &str, history_file: Option<&Path>) -> io::Result<Self> {
        let mut s = Self {
            inner: Arc::new(InnerHackshell {
                ctx,
                commands: Default::default(),
                prompt: prompt.to_string(),
                rl: Mutex::new(Readline::new(stdin(), history_file).await?),
                pool: Default::default(),
            }),
        };

        s.add_command(Sleep {}).await;
        s.add_command(Exit {}).await;

        Ok(s)
    }

    pub async fn add_command(&mut self, command: impl Command<C> + 'static) {
        let c = Arc::new(command);

        for cmd in c.commands().iter() {
            self.inner
                .commands
                .write()
                .await
                .insert(cmd.to_string(), c.clone());
        }
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
        let event = self
            .inner
            .rl
            .lock()
            .await
            .readline(&self.inner.prompt)
            .await
            .to_estring()?;

        match event {
            Event::Line(line) => {
                let cmd = shlex::Shlex::new(&line).collect::<Vec<String>>();

                if cmd.is_empty() {
                    return Ok(());
                }

                self.inner
                    .commands
                    .read()
                    .await
                    .get(&cmd[0])
                    .ok_or("Command not found")?
                    .run(self, &cmd, &self.inner.ctx)
                    .await
                    .to_estring()?;
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
