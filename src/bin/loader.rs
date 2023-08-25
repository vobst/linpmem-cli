/* SPDX-FileCopyrightText: © 2023 Valentin Obst <legal@eb9f.de>
 * SPDX-License-Identifier: MIT
 */

use clap::Parser;
use pmem::LoaderCli;
use std::process;

fn main() {
    let cli = LoaderCli::parse();

    if let Err(err) = pmem::insmod::run(&cli.args) {
        eprintln!("Application error: {err}");
        process::exit(1);
    }
}
