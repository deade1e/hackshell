use std::error::Error;

use hackshell::{Command, CommandResult, Hackshell, error::HackshellError};

struct Counter {
    num: u64,
}

impl Command for Counter {
    fn commands(&self) -> &'static [&'static str] {
        &["counter"]
    }

    fn help(&self) -> &'static str {
        "This is a non-default command installed by the Hackshell consumer. It simply increments an internal counter"
    }

    fn run(&mut self, _s: &Hackshell, _cmd: &[&str]) -> CommandResult {
        self.num += 1;

        println!("The counter is now: {}\r", self.num);

        Ok(None)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let s = Hackshell::new("counter> ")?;

    s.add_command(Counter { num: 0 });

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
