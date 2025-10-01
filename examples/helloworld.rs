use std::error::Error;

use hackshell::{Command, CommandResult, Hackshell, error::HackshellError};

struct HelloWorld {}

impl Command for HelloWorld {
    fn commands(&self) -> &'static [&'static str] {
        &["helloworld"]
    }

    fn help(&self) -> &'static str {
        "This is a non-default command installed by the Hackshell consumer. It simply prints Hello, World."
    }

    fn run(&self, _s: &Hackshell, _cmd: &[&str]) -> CommandResult {
        println!("Hello, World!");
        Ok(None)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let s = Hackshell::new("helloworld> ")?;

    s.add_command(HelloWorld {});

    loop {
        match s.run() {
            Ok(_) => {}
            Err(e) => {
                if matches!(e, HackshellError::Eof)
                    || matches!(e, HackshellError::Interrupted)
                    || matches!(e, HackshellError::Exit)
                {
                    break;
                }

                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
