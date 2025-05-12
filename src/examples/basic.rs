use std::path::Path;
use hackshell::Hackshell;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new shell with a custom context (in this case just ())
    let shell = Hackshell::new((), "hackshell> ", Some(Path::new("history.txt"))).await?;
    
    // Enter the shell loop
    loop {
        match shell.run().await {
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
