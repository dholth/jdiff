use jdiff::patchy;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    // [0] is the command name
    let before = &args[1]; // previous version
    let after = &args[2]; // current version
    let patches = &args[3]; // patches file

    if let Err(e) = patchy(before, after, patches) {
        eprintln!("{}", e);
    }
}
