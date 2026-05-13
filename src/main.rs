mod cli;

use clap::Parser;
use cli::Cli;
#[allow(unused_imports)]
use git_metadata::exe;
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

fn run(_cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}

#[allow(dead_code)]
fn atty_stdin() -> bool {
    todo!()
}

fn parse_generate_man_flag() -> Option<PathBuf> {
    todo!()
}

#[allow(dead_code)]
fn default_man_dir() -> PathBuf {
    todo!()
}

fn generate_man_page(_output_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}

#[allow(dead_code)]
fn manpath_covers(_dir: &std::path::Path) -> bool {
    todo!()
}
