use std::{
    collections::HashMap, io, path::Path, rc::Rc, sync::{atomic::AtomicBool, Arc}
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

pub trait Command<C>: 'static {
    fn commands(&self) -> &'static [&'static str];

    fn help(&self) -> &'static str;

    fn run(&self, s: &mut Hackshell<C>, cmd: &[String]) -> Result<(), String>;
}

struct InnerHackshell<C> {
    ctx: C,
    commands: HashMap<String, Rc<dyn Command<C>>>,
    env: HashMap<String, String>,
    pool: TaskPool,
    prompt: String,
    rl: Readline,
}

pub struct Hackshell<C> {
    inner: InnerHackshell<C>,
}

impl<C: 'static> Hackshell<C> {
    pub fn new(ctx: C, prompt: &str, history_file: Option<&Path>) -> io::Result<Self> {
        let mut s = Self {
            inner: InnerHackshell {
                ctx,
                commands: Default::default(),
                env: Default::default(),
                pool: Default::default(),
                prompt: prompt.to_string(),
                rl: Readline::new(history_file)?,
            },
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

    pub fn add_command<D: Command<C> + 'static>(&mut self, command: D) -> &mut Self {
        let c = Rc::new(command);

        for cmd in c.commands().iter() {
            self.inner.commands.insert(cmd.to_string(), c.clone());
        }

        self
    }

    pub fn get_ctx(&self) -> &C {
        &self.inner.ctx
    }

    pub fn get_mut_ctx(&mut self) -> &mut C {
        &mut self.inner.ctx
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

    pub fn get_commands(&self) -> Vec<Rc<dyn Command<C>>> {
        self.inner.commands.iter().map(|c| c.1.clone()).collect()
    }

    pub fn env(&self) -> HashMap<String, String> {
        self.inner.env.clone()
    }

    pub fn get_var(&self, n: &str) -> Option<String> {
        self.inner.env.get(&n.to_lowercase()).cloned()
    }

    pub fn set_var(&mut self, n: &str, v: &str) {
        self.inner.env.insert(n.to_lowercase(), v.to_string());
    }

    pub fn unset_var(&mut self, n: &str) {
        self.inner.env.remove(n);
    }

    pub fn feed_slice(&mut self, cmd: &[String]) -> Result<(), String> {
        if cmd.is_empty() {
            return Ok(());
        }

        let command = self.inner.commands.get(&cmd[0]).cloned();

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

    pub fn feed_line(&mut self, line: &str) -> Result<(), String> {
        let cmd = shlex::Shlex::new(line).collect::<Vec<String>>();
        self.feed_slice(&cmd)
    }

    pub fn run(&mut self) -> Result<(), String> {
        let event = self.inner.rl.readline(&self.inner.prompt).to_estring()?;

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
