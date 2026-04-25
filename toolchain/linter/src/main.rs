use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("luwlinter: usage: luwlinter <files...>");
        process::exit(1);
    }
    eprintln!("luwlinter: linting not yet implemented");
    process::exit(0);
}
