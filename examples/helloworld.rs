use std::error::Error;

use hackshell::{Command, CommandResult, Hackshell};

struct HelloWorld {}

impl Command<()> for HelloWorld {
    fn commands(&self) -> &'static [&'static str] {
        &["helloworld"]
    }

    fn help(&self) -> &'static str {
        "This is a non-default command installed by the Hackshell consumer. It simply prints Hello, World."
    }

    fn run(&self, _s: &Hackshell<()>, _cmd: &[String]) -> CommandResult {
        println!("Hello, World!");
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let s = Hackshell::new((), "helloworld> ")?;

    s.add_command(HelloWorld {});

    loop {
        s.run()?;
    }
}
