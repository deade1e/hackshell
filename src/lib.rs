use std::{
    collections::HashMap,
    io,
    path::Path,
    sync::{Arc, Mutex, MutexGuard, RwLock, atomic::AtomicBool},
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

pub trait Command<C>: Send + Sync + 'static {
    fn commands(&self) -> &'static [&'static str];

    fn help(&self) -> &'static str;

    fn run(&self, s: &Hackshell<C>, cmd: &[String]) -> Result<(), String>;
}

struct InnerHackshell<C> {
    ctx: Mutex<C>,
    commands: RwLock<HashMap<String, Arc<dyn Command<C>>>>,
    env: RwLock<HashMap<String, String>>,
    pool: TaskPool,
    prompt: RwLock<String>,
    rl: Mutex<Readline>,
}

pub struct Hackshell<C> {
    inner: Arc<InnerHackshell<C>>,
}

impl<C> Clone for Hackshell<C> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<C: 'static> Hackshell<C> {
    pub fn new(ctx: C, prompt: &str, history_file: Option<&Path>) -> io::Result<Self> {
        let mut s = Self {
            inner: Arc::new(InnerHackshell {
                ctx: Mutex::new(ctx),
                commands: Default::default(),
                env: Default::default(),
                pool: Default::default(),
                prompt: RwLock::new(prompt.to_string()),
                rl: Mutex::new(Readline::new(history_file)?),
            }),
        };

        s.add_command(Env {})
            .add_command(Get {})
            .add_command(Set {})
            .add_command(Unset {})
            .add_command(Help {})
            .add_command(Sleep {})
            .add_command(Exit {})
            .add_command(Task {});

        Ok(s)
    }

    pub fn add_command<D: Command<C> + 'static>(&self, command: D) -> &Self {
        let c = Arc::new(command);

        for cmd in c.commands().iter() {
            self.inner
                .commands
                .write()
                .unwrap()
                .insert(cmd.to_string(), c.clone());
        }

        self
    }

    pub fn get_ctx(&self) -> MutexGuard<'_, C> {
        self.inner.ctx.lock().unwrap()
    }

    pub fn spawn<F: Fn(Arc<AtomicBool>) + Send + 'static>(&self, name: &str, func: F) {
        self.inner.pool.spawn(name, func);
    }

    pub fn terminate(&self, name: &str) -> Result<(), String> {
        self.inner.pool.remove(name)
    }

    pub fn wait(&self, name: &str) {
        self.inner.pool.wait(name);
    }

    pub fn get_tasks(&self) -> Vec<TaskMetadata> {
        self.inner.pool.get_all()
    }

    pub fn get_commands(&self) -> Vec<Arc<dyn Command<C>>> {
        self.inner
            .commands
            .read()
            .unwrap()
            .iter()
            .map(|c| c.1.clone())
            .collect()
    }

    pub fn env(&self) -> HashMap<String, String> {
        self.inner.env.read().unwrap().clone()
    }

    pub fn get_var(&self, n: &str) -> Option<String> {
        self.inner
            .env
            .read()
            .unwrap()
            .get(&n.to_lowercase())
            .cloned()
    }

    pub fn set_var(&self, n: &str, v: &str) {
        self.inner
            .env
            .write()
            .unwrap()
            .insert(n.to_lowercase(), v.to_string());
    }

    pub fn unset_var(&self, n: &str) {
        self.inner.env.write().unwrap().remove(n);
    }

    pub fn feed_slice(&self, cmd: &[String]) -> Result<(), String> {
        if cmd.is_empty() {
            return Ok(());
        }

        let command = self.inner.commands.read().unwrap().get(&cmd[0]).cloned();

        match command {
            Some(c) => {
                if let Err(e) = c.run(self, cmd) {
                    if e == "exit" {
                        return Err(e);
                    }

                    eprintln!("{}", e);
                }
            }
            None => {
                eprintln!("Command not found");
            }
        }

        Ok(())
    }

    pub fn feed_line(&self, line: &str) -> Result<(), String> {
        let cmd = shlex::Shlex::new(line).collect::<Vec<String>>();
        self.feed_slice(&cmd)
    }

    pub fn run(&self) -> Result<(), String> {
        let event = self
            .inner
            .rl
            .lock()
            .unwrap()
            .readline(&self.inner.prompt.read().unwrap())
            .to_estring()?;

        match event {
            Event::Line(line) => {
                return self.feed_line(&line);
            }
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
