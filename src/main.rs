/* SPDX-FileCopyrightText: © 2023 Valentin Obst <legal@bpfvol3.de>
 * SPDX-License-Identifier: MIT
 */
use clap::Parser;
use pmem::Cli;
use std::process;

fn main() {
    let cli = Cli::parse();

    if let Err(err) = pmem::run(&cli) {
        eprintln!("Application error: {err}");
        process::exit(1);
    }
}
