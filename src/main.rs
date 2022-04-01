use jdiff::{apply, patchy};

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    left: PathBuf,

    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    right: PathBuf,

    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    patchset: PathBuf,

    #[clap(short, long)]
    indent: bool,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Patch {},
}

fn main() {
    let cli = Cli::parse();

    let before = cli.left; // previous version
    let after = cli.right; // current version
    let patches = cli.patchset; // patches file

    match &cli.command {
        Some(Commands::Patch {}) => {
            if let Err(e) = apply(after.as_path(), patches.as_path(), cli.indent) {
                eprintln!("{}", e);
            }
        }
        None => {
            if let Err(e) = patchy(
                before.as_path(),
                after.as_path(),
                patches.as_path(),
                cli.indent,
            ) {
                eprintln!("{}", e);
            }
        }
    }
}
