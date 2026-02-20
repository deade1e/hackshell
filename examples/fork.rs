use hackshell::{Command, CommandResult, Hackshell, error::HackshellError};

/// A command that forks into a subshell with a custom prompt.
/// Usage: subshell [prompt]
struct SubshellCommand;

impl Command for SubshellCommand {
    fn commands(&self) -> &'static [&'static str] {
        &["subshell", "fork"]
    }

    fn help(&self) -> &'static str {
        "Fork into a subshell. Usage: subshell [prompt]"
    }

    fn run(&self, shell: &Hackshell, cmd: &[&str]) -> CommandResult {
        // Determine the prompt for the subshell
        let prompt = if cmd.len() > 1 {
            format!("{}> ", cmd[1])
        } else {
            "sub> ".to_string()
        };

        // Fork the shell - this creates a child shell with inherited environment
        let subshell = shell.fork(&prompt)?;

        println!("Entering subshell (type 'exit' to return)...");
        println!("Environment variables have been inherited from parent.");

        // Run the subshell loop
        loop {
            match subshell.run() {
                Ok(output) => {
                    if let Some(text) = output {
                        println!("{}", text);
                    }
                }
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

        println!("Exited subshell, returning to parent.");

        Ok(None)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the main shell
    let shell = Hackshell::new("main> ")?;

    // Add the subshell command
    shell.add_command(SubshellCommand);

    println!("Hackshell fork() example");
    println!("========================");
    println!();
    println!("This example demonstrates the fork() method which creates");
    println!("a child shell that inherits the parent's environment.");
    println!();
    println!("Try these commands:");
    println!("  set name Alice     - Set a variable in the main shell");
    println!("  subshell           - Fork into a subshell");
    println!("  get name           - See the inherited variable");
    println!("  set name Bob       - Modify in subshell (won't affect parent)");
    println!("  exit               - Return to parent shell");
    println!("  get name           - Verify parent's value is unchanged");
    println!();

    // Main shell loop
    loop {
        match shell.run() {
            Ok(output) => {
                if let Some(text) = output {
                    println!("{}", text);
                }
            }
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

    println!("Goodbye!");

    Ok(())
}
