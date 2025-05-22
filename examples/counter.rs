use hackshell::{Command, Hackshell, error::MapErrToString};

struct Counter {}

impl Command<u64> for Counter {
    fn commands(&self) -> &'static [&'static str] {
        &["counter"]
    }

    fn help(&self) -> &'static str {
        "This is a non-default command installed by the Hackshell consumer. It simply increments an internal counter"
    }

    fn run(&self, s: &mut Hackshell<u64>, _cmd: &[String]) -> Result<(), String> {
        
        let num = s.get_mut_ctx();
        *num += 1;

        println!("The counter is now: {}\r", num);

        Ok(())
    }
}

fn main() -> Result<(), String> {
    let mut s = Hackshell::new(0u64, "hackshell> ", None).to_estring()?;

    s.add_command(Counter {});

    loop {
        s.run().to_estring()?;
    }
}
