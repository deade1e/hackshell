use hackshell::Hackshell;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new shell with a custom context (in this case just ())
    let mut shell = Hackshell::<()>::new("hackshell> ", Some(Path::new("history.txt")))?;

    // Enter the shell loop
    loop {
        match shell.run() {
            Ok(_) => {}
            Err(e) => {
                if e == "EOF" || e == "CTRLC" || e == "exit" {
                    break;
                }
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
