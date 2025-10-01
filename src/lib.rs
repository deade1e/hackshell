use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock, atomic::AtomicBool},
};

use crate::{
    error::{HackshellError, HackshellResult},
    taskpool::TaskOutput,
};

mod commands;
pub mod error;
pub mod taskpool;

use commands::{
    env::Env, exit::Exit, get::Get, help::Help, set::Set, sleep::Sleep, task::Task, unset::Unset,
};
use rustyline::{DefaultEditor, error::ReadlineError};
use taskpool::{TaskMetadata, TaskPool};

pub type CommandResult =
    std::result::Result<Option<String>, Box<dyn std::error::Error + Send + Sync + 'static>>;

pub trait Command: Send + Sync + 'static {
    fn commands(&self) -> &'static [&'static str];

    fn help(&self) -> &'static str;

    fn run(&self, s: &Hackshell, cmd: &[&str]) -> CommandResult;
}

type InnerCommandEntry = dyn Command;

#[derive(Clone)]
pub struct CommandEntry {
    inner: Arc<InnerCommandEntry>,
}

impl CommandEntry {
    fn new(c: impl Command) -> Self {
        Self { inner: Arc::new(c) }
    }

    fn commands(&self) -> &'static [&'static str] {
        self.inner.commands()
    }

    fn help(&self) -> &'static str {
        self.inner.help()
    }

    fn run(&self, s: &Hackshell, cmd: &[&str]) -> CommandResult {
        self.inner.run(s, cmd)
    }
}

type Commands = HashMap<String, CommandEntry>;
type Environment = HashMap<String, String>;

struct InnerHackshell {
    commands: RwLock<Commands>,
    env: RwLock<Environment>,
    pool: TaskPool,
    prompt: RwLock<String>,
    history_file: RwLock<Option<PathBuf>>,
    rl: Mutex<DefaultEditor>,
}

#[derive(Clone)]
pub struct Hackshell {
    inner: Arc<InnerHackshell>,
}

impl Hackshell {
    pub fn new(prompt: &str) -> HackshellResult<Self> {
        let rl = DefaultEditor::new()?;

        let s = Self {
            inner: Arc::new(InnerHackshell {
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

    pub fn set_history_file<P: AsRef<Path>>(&self, path: P) -> HackshellResult<()> {
        let mut hf = self.inner.history_file.write().unwrap();
        *hf = Some(path.as_ref().to_path_buf());

        let res = self.inner.rl.lock().unwrap().load_history(&path);

        if let Err(ReadlineError::Io(ref e)) = res {
            if matches!(e.kind(), std::io::ErrorKind::NotFound) {
                return Ok(());
            }
        }

        res?;

        Ok(())
    }

    pub fn add_command(&self, command: impl Command) -> &Self {
        let ce = CommandEntry::new(command);

        for cmd in ce.commands().iter() {
            self.inner
                .commands
                .write()
                .unwrap()
                .insert(cmd.to_string(), ce.clone());
        }

        self
    }

    pub fn spawn<F>(&self, name: &str, func: F)
    where
        F: FnOnce(Arc<AtomicBool>) -> TaskOutput + Send + 'static,
    {
        self.inner.pool.spawn(name, func);
    }

    #[cfg(feature = "async")]
    pub fn spawn_async<F>(&self, name: &str, func: F)
    where
        F: Future<Output = TaskOutput> + Send + Sync + 'static,
    {
        self.inner.pool.spawn_async(name, func);
    }

    pub fn terminate(&self, name: &str) -> HackshellResult<()> {
        self.inner.pool.remove(name)
    }

    pub fn join(&self, name: &str) -> HackshellResult<TaskOutput> {
        self.inner.pool.join(name)
    }

    #[cfg(feature = "async")]
    pub async fn join_async(&self, name: &str) -> HackshellResult<TaskOutput> {
        self.inner.pool.join_async(name).await
    }

    pub fn get_tasks(&self) -> Vec<TaskMetadata> {
        self.inner.pool.get_all()
    }

    pub fn get_commands(&self) -> Vec<CommandEntry> {
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

    pub fn feed_slice(&self, cmd: &[&str]) -> HackshellResult<Option<String>> {
        if cmd.is_empty() {
            return Ok(None);
        }

        let command = self.inner.commands.read().unwrap().get(cmd[0]).cloned();

        match command {
            Some(c) => {
                return Ok(c.run(self, cmd)?);
            }
            None => Err(HackshellError::CommandNotFound),
        }
    }

    pub fn feed_line(&self, line: &str) -> HackshellResult<Option<String>> {
        let cmd = shlex::Shlex::new(line).collect::<Vec<String>>();
        let cmd_refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
        self.feed_slice(&cmd_refs)
    }

    pub fn run(&self) -> HackshellResult<Option<String>> {
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
                return Err(e.into());
            }
        }
    }
}
