use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::{
    io::{self},
    sync::{Mutex, RwLock},
};

mod commands;
pub mod error;
mod readline;
mod taskpool;

use crate::readline::{Event, Readline};
use commands::{
    env::Env, exit::Exit, get::Get, help::Help, set::Set, sleep::Sleep, task::Task, unset::Unset,
};
use error::MapErrToString;
use taskpool::{TaskMetadata, TaskPool};

#[async_trait::async_trait]
pub trait Command<C>: Send + Sync + 'static {
    fn commands(&self) -> &'static [&'static str];

    fn help(&self) -> &'static str;

    async fn run(&self, s: &Hackshell<C>, cmd: &[String], ctx: &C) -> Result<(), String>;
}

struct InnerHackshell<C> {
    ctx: C,
    commands: RwLock<HashMap<String, Arc<dyn Command<C>>>>,
    env: RwLock<HashMap<String, String>>,
    pool: TaskPool,
    prompt: String,
    rl: Mutex<Readline>,
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

impl<C: Send + Sync + 'static> Hackshell<C> {
    pub async fn new(ctx: C, prompt: &str, history_file: Option<&Path>) -> io::Result<Self> {
        let s = Self {
            inner: Arc::new(InnerHackshell {
                ctx,
                commands: Default::default(),
                env: Default::default(),
                pool: Default::default(),
                prompt: prompt.to_string(),
                rl: Mutex::new(Readline::new(history_file).await?),
            }),
        };

        s.add_command(Env {})
            .await
            .add_command(Get {})
            .await
            .add_command(Set {})
            .await
            .add_command(Unset {})
            .await
            .add_command(Help {})
            .await
            .add_command(Sleep {})
            .await
            .add_command(Exit {})
            .await
            .add_command(Task {})
            .await;

        Ok(s)
    }

    pub async fn add_command(&self, command: impl Command<C> + 'static) -> Self {
        let c = Arc::new(command);

        for cmd in c.commands().iter() {
            self.inner
                .commands
                .write()
                .await
                .insert(cmd.to_string(), c.clone());
        }

        self.clone()
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

    pub async fn get_tasks(&self) -> Vec<TaskMetadata> {
        self.inner.pool.get_all().await
    }

    pub async fn get_commands(&self) -> Vec<Arc<dyn Command<C>>> {
        self.inner
            .commands
            .read()
            .await
            .iter()
            .map(|c| c.1.clone())
            .collect()
    }

    pub async fn env(&self) -> HashMap<String, String> {
        self.inner.env.read().await.clone()
    }

    pub async fn get_var(&self, n: &str) -> Option<String> {
        self.inner.env.read().await.get(&n.to_lowercase()).cloned()
    }

    pub async fn set_var(&self, n: &str, v: &str) {
        self.inner
            .env
            .write()
            .await
            .insert(n.to_lowercase(), v.to_string());
    }

    pub async fn unset_var(&self, n: &str) {
        self.inner.env.write().await.remove(n);
    }

    pub async fn feed_line(&self, line: &str) -> Result<(), String> {
        let cmd = shlex::Shlex::new(line).collect::<Vec<String>>();

        if cmd.is_empty() {
            return Ok(());
        }

        match self.inner.commands.read().await.get(&cmd[0]) {
            Some(c) => {
                if let Err(e) = c.run(self, &cmd, &self.inner.ctx).await {
                    eprintln!("{}", e);
                }
            }
            None => {
                eprintln!("Command not found");
            }
        }

        Ok(())
    }

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
            Event::Line(line) => {return self.feed_line(&line).await;}
            Event::Ctrlc => {
                return Err("CTRLC".to_string());
            }
            Event::Eof => {
                return Err("EOF".to_string());
            }
            Event::Tab => {}
        }

        Ok(())
    }
}
