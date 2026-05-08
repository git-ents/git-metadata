mod cli;
mod exe;

use clap::{CommandFactory, Parser};
use cli::Cli;
use std::path::PathBuf;
use std::process;

fn main() {
    if let Some(dir) = parse_generate_man_flag() {
        if let Err(e) = generate_man_page(dir) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
        return;
    }

    let cli = Cli::parse();

    if let Err(e) = run(&cli) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}

fn atty_stdin() -> bool {
    todo!()
}

fn parse_generate_man_flag() -> Option<PathBuf> {
    todo!()
}

fn default_man_dir() -> PathBuf {
    todo!()
}

fn generate_man_page(output_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}

fn manpath_covers(dir: &std::path::Path) -> bool {
    todo!()
}
