use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("luwfmt: usage: luwfmt <files...>");
        process::exit(1);
    }
    eprintln!("luwfmt: formatting not yet implemented");
    process::exit(0);
}
