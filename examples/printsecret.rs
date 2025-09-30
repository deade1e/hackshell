use std::error::Error;

use hackshell::{Command, CommandResult, Hackshell, error::HackshellError};

struct MyContext {
    secret: String,
}

struct PrintSecret {}

impl Command<MyContext> for PrintSecret {
    fn commands(&self) -> &'static [&'static str] {
        &["printsecret"]
    }

    fn help(&self) -> &'static str {
        "This is a non-default command installed by the Hackshell consumer. It prints a variable inside the passed context."
    }

    fn run(&self, s: &Hackshell<MyContext>, _cmd: &[String]) -> CommandResult {
        println!("{}", s.get_ctx().secret);
        Ok(None)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let ctx = MyContext {
        secret: "It rains red in some parts of the world".to_string(),
    };

    let s = Hackshell::new(ctx, "printsecret> ")?;

    s.add_command(PrintSecret {});

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
