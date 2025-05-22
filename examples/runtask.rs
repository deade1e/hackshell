use std::{thread::sleep, time::Duration};

use hackshell::{Command, Hackshell, error::MapErrToString};

struct RunTask {}

impl Command<()> for RunTask {
    fn commands(&self) -> &'static [&'static str] {
        &["runtask"]
    }

    fn help(&self) -> &'static str {
        "This is a non-default command installed by the Hackshell consumer. It simply spawns a task that lasts n seconds in the background."
    }

    fn run(&self, s: &mut Hackshell<()>, cmd: &[String]) -> Result<(), String> {
        if cmd.len() < 2 {
            return Err("Syntax: runtask <nsecs>".to_string());
        }

        // .to_estring() comes from the hackshell::error::MapErrToString trait
        let n = cmd[1].parse::<u64>().to_estring()?;

        s.spawn("runtask", move |run| {
            let mut c = 10;
            println!("RunTask started. Use the `task` command to see it! This task will finish after 10 prints or can be terminated by issuing `task -t runtask`.\r");

            while c > 0 && run.load(std::sync::atomic::Ordering::Relaxed) {
                println!("RunTask is running!\r");
                sleep(Duration::from_secs(n));
                c -= 1;
            }
        });

        Ok(())
    }
}

fn main() -> Result<(), String> {
    let mut s = Hackshell::new((), "hackshell> ", None).to_estring()?;

    s.add_command(RunTask {});

    loop {
        s.run().to_estring()?;
    }
}
