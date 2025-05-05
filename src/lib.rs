use std::{collections::HashMap, path::Path, sync::Arc};

use readline::{Event, Readline};
use tokio::io::{self, stdin, AsyncRead, Stdin};

mod commands;

use commands::{exit::Exit, sleep::Sleep};

// struct Task {
//     name: String,
//     started: chrono::DateTime<chrono::Utc>,
//     alive: bool,
//     terminate: watch::Sender<()>,
// }

// pub enum Event {
//     Line(String),
//     CTRLC,
//     EOF, // CTRL + d
//     TAB,
//     SUB, // CTRL + z
// }

pub trait InputProvider {
    async fn read_line(&self) -> io::Result<Event>;
}

#[async_trait::async_trait]
pub trait Command<C>: Send + Sync + 'static {
    fn commands(&self) -> &'static [&'static str];
    
    fn help(&self) -> &'static str;

    async fn run(&self, s: &Hackshell<C>, cmd: &[String], ctx: &C) -> Result<(), String>;
}

struct InnerHackshell<C> {
    ctx: C,
    rl: Readline<Stdin>,
    commands: HashMap<String, Arc<dyn Command<C>>>,
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
                commands: Default::default(), // tasks: Default::default()
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
}

impl<C: 'static> Hackshell<C> {
    pub async fn run(&self) -> Result<(), String> {
        let line = self.inner.rl.run().await.map_err(|e| e.to_string())?;

        match line {
            Event::Line(line) => {
                let lexer = shlex::Shlex::new(&line);
                let cmd: Vec<String> = lexer.collect();

                if let Some(c) = self.inner.commands.get(&cmd[0]) {
                    c.run(&self, &cmd, &self.inner.ctx)
                        .await
                        .map_err(|e| e.to_string())?;
                } else {
                    eprintln!("Command not found");
                }
            }
            Event::CTRLC => {}
            Event::EOF => {}
            Event::TAB => {}
            _ => {}
        }

        Ok(())
    }
}
