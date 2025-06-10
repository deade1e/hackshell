use hackshell::{Command, Hackshell, error::MapErrToString};

struct HelloWorld {}

impl Command<()> for HelloWorld {
    fn commands(&self) -> &'static [&'static str] {
        &["helloworld"]
    }

    fn help(&self) -> &'static str {
        "This is a non-default command installed by the Hackshell consumer. It simply prints Hello, World."
    }

    fn run(&self, _s: &Hackshell<()>, _cmd: &[String]) -> Result<(), String> {
        println!("Hello, World!");
        Ok(())
    }
}

fn main() -> Result<(), String> {
    let mut s = Hackshell::new((), "hackshell> ", None).to_estring()?;

    s.add_command(HelloWorld {});

    loop {
        s.run().to_estring()?;
    }
}
