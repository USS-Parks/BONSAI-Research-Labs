use std::env;
use std::io::{self, Write};
use std::process::{self, Command};
use std::thread;
use std::time::Duration;

fn main() {
    if env::args().nth(1).as_deref() == Some("--child") {
        thread::sleep(Duration::from_millis(750));
        return;
    }
    let executable = env::current_exe().expect("current executable");
    let mut child = Command::new(executable)
        .arg("--child")
        .spawn()
        .expect("spawn child");
    println!("{} {}", process::id(), child.id());
    io::stdout().flush().expect("flush process identities");
    child.wait().expect("wait child");
}
