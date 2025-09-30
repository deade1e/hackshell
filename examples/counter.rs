use std::error::Error;

use hackshell::{Command, CommandResult, Hackshell, error::HackshellError};

struct Counter {}

impl Command<u64> for Counter {
    fn commands(&self) -> &'static [&'static str] {
        &["counter"]
    }

    fn help(&self) -> &'static str {
        "This is a non-default command installed by the Hackshell consumer. It simply increments an internal counter"
    }

    fn run(&self, s: &Hackshell<u64>, _cmd: &[String]) -> CommandResult {
        let mut num = s.get_ctx();
        *(num) += 1;

        println!("The counter is now: {}\r", num);

        Ok(None)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let s = Hackshell::new(0u64, "counter> ")?;

    s.add_command(Counter {});

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
