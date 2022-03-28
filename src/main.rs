use jdiff::{apply, patchy};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // [0] is the command name
    let before = &args[1]; // previous version
    let after = &args[2]; // current version
    let patches = &args[3]; // patches file

    if before == "apply" {
        if let Err(e) = apply(after, patches) {
            eprintln!("{}", e);
        }
    } else if let Err(e) = patchy(before, after, patches) {
        eprintln!("{}", e);
    }
}
