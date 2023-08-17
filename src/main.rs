use pmem::Cli;
use clap::Parser;
use std::process::exit;

fn main() {
    let cli = Cli::parse();

    if let Err(err) = pmem::run(&cli) {
        eprintln!("Application error: {err}");
        exit(1);
    }
}
