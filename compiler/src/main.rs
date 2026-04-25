use std::env;
use std::process;

use luwi_script::driver;

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Err(e) = driver::run(args) {
        eprintln!("\x1b[31merror\x1b[0m: {e}");
        process::exit(1);
    }
}
