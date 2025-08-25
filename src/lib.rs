use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard, RwLock, atomic::AtomicBool},
};

use crate::error::{HackshellError, Result};

mod commands;
pub mod error;
mod taskpool;

use commands::{
    env::Env, exit::Exit, get::Get, help::Help, set::Set, sleep::Sleep, task::Task, unset::Unset,
};
use rustyline::{DefaultEditor, error::ReadlineError};
use taskpool::{TaskMetadata, TaskPool};

pub trait Command<C>: Send + Sync + 'static {
    fn commands(&self) -> &'static [&'static str];

    fn help(&self) -> &'static str;

    fn run(&self, s: &Hackshell<C>, cmd: &[String]) -> CommandResult;
}

pub type CommandResult =
    std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;

struct InnerHackshell<C> {
    ctx: Mutex<C>,
    commands: RwLock<HashMap<String, Arc<dyn Command<C>>>>,
    env: RwLock<HashMap<String, String>>,
    pool: TaskPool,
    prompt: RwLock<String>,
    history_file: RwLock<Option<PathBuf>>,
    rl: Mutex<DefaultEditor>,
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

impl<C: 'static> Hackshell<C> {
    pub fn new(ctx: C, prompt: &str) -> Result<Self> {
        let rl = DefaultEditor::new()?;

        let s = Self {
            inner: Arc::new(InnerHackshell {
                ctx: Mutex::new(ctx),
                commands: Default::default(),
                env: Default::default(),
                pool: Default::default(),
                prompt: RwLock::new(prompt.to_string()),
                history_file: Default::default(),
                rl: Mutex::new(rl),
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

    pub fn set_history_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut hf = self.inner.history_file.write().unwrap();
        *hf = Some(path.as_ref().to_path_buf());

        self.inner.rl.lock().unwrap().load_history(&path)?;

        Ok(())
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

    pub fn spawn<F: FnOnce(Arc<AtomicBool>) + Send + 'static>(&self, name: &str, func: F) {
        self.inner.pool.spawn(name, func);
    }

    pub fn terminate(&self, name: &str) -> Result<()> {
        self.inner.pool.remove(name)
    }

    pub fn wait(&self, name: &str) -> Result<()> {
        self.inner.pool.wait(name)
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

    pub fn feed_slice(&self, cmd: &[String]) -> Result<()> {
        if cmd.is_empty() {
            return Ok(());
        }

        let command = self.inner.commands.read().unwrap().get(&cmd[0]).cloned();

        match command {
            Some(c) => {
                if let Err(e) = c.run(self, cmd) {
                    if let Some(HackshellError::Exit) = e.downcast_ref::<HackshellError>() {
                        return Err(e.into());
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

    pub fn feed_line(&self, line: &str) -> Result<()> {
        let cmd = shlex::Shlex::new(line).collect::<Vec<String>>();
        self.feed_slice(&cmd)
    }

    pub fn run(&self) -> Result<()> {
        let mut rl = self.inner.rl.lock().unwrap();
        let readline = rl.readline(&*self.inner.prompt.read().unwrap());

        match readline {
            Ok(line) => {
                if let Some(hfile) = self.inner.history_file.read().unwrap().as_ref() {
                    rl.add_history_entry(&line)?;
                    rl.save_history(hfile)?;
                }

                return self.feed_line(&line);
            }
            Err(e)
                if matches!(e, ReadlineError::Interrupted) || matches!(e, ReadlineError::Eof) =>
            {
                return Err(e.into());
            }

            Err(e) => {
                eprintln!("{}", e);
            }
        }

        Ok(())
    }
}
