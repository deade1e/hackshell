use hackshell::{Hackshell, error::HackshellError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new shell with a custom context (in this case just ())
    let shell = Hackshell::<()>::new((), "basic> ")?;
    shell.set_history_file("history.txt")?;

    // Enter the shell loop
    loop {
        match shell.run() {
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
