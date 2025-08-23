use hackshell::{Hackshell, error::HackshellError};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new shell with a custom context (in this case just ())
    let shell = Hackshell::<()>::new((), "basic> ", Some(Path::new("history.txt")))?;

    // Enter the shell loop
    loop {
        match shell.run() {
            Ok(_) => {}
            Err(e) => {
                if matches!(e, HackshellError::Eof)
                    || matches!(e, HackshellError::Interrupted)
                    || matches!(e, HackshellError::ShellExit)
                {
                    break;
                }

                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
