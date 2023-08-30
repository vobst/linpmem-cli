/* SPDX-FileCopyrightText: © 2023 Valentin Obst <legal@eb9f.de>
 * SPDX-License-Identifier: MIT
 */

use clap::Parser;
use env_logger;
use log;
use pmem::LoaderCli;
use std::process;

fn main() {
    let cli = LoaderCli::parse();

    env_logger::Builder::new()
        .filter_level(if cli.args.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Error
        })
        .init();

    if let Err(err) = pmem::insmod::run(&cli.args) {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}
