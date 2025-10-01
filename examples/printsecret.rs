use std::error::Error;

use hackshell::{Command, CommandResult, Hackshell, error::HackshellError};

struct PrintSecret {
    secret: String,
}

impl Command for PrintSecret {
    fn commands(&self) -> &'static [&'static str] {
        &["printsecret"]
    }

    fn help(&self) -> &'static str {
        "This is a non-default command installed by the Hackshell consumer. It prints a variable inside the passed context."
    }

    fn run(&self, _s: &Hackshell, _cmd: &[&str]) -> CommandResult {
        println!("{}", self.secret);
        Ok(None)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let s = Hackshell::new("printsecret> ")?;

    s.add_command(PrintSecret {
        secret: "It rains red in some parts of the world".to_string(),
    });

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
