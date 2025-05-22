use hackshell::{Command, Hackshell, error::MapErrToString};

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

    fn run(&self, s: &mut Hackshell<MyContext>, _cmd: &[String]) -> Result<(), String> {
        println!("{}", s.get_ctx().unwrap().secret);
        Ok(())
    }
}

fn main() -> Result<(), String> {
    let ctx = MyContext {
        secret: "It rains red in some parts of the world".to_string(),
    };

    let mut s = Hackshell::new("hackshell> ", None).to_estring()?;

    s.set_ctx(ctx);

    s.add_command(PrintSecret {});

    loop {
        s.run().to_estring()?;
    }
}
