/* SPDX-FileCopyrightText: © 2023 Valentin Obst <legal@bpfvol3.de>
 * SPDX-License-Identifier: MIT
 */

mod cli;
pub mod insmod;
mod ioctl;
mod utils;

use crate::cli::Subcommands;
pub use crate::cli::{Cli, LoaderCli};
use crate::ioctl::{Driver, IOCtlCmd};
use std::error::Error;

pub fn run(cli: &Cli) -> Result<(), Box<dyn Error>> {
    if let Some(subcommand) = &cli.subcommand {
        return match subcommand {
            Subcommands::Insmod(insmod_cli) => insmod::run(insmod_cli),
        };
    }

    let cmd = IOCtlCmd::from_cli(cli)?;
    let drv = Driver::build()?;

    match cmd {
        IOCtlCmd::Cr3(pid) => Ok(drv.cr3(pid)?),
        IOCtlCmd::VtoP(virt_address, pid) => Ok(drv.v_to_p(virt_address, pid)?),
        IOCtlCmd::ReadPhys(address, mode, size) => {
            Ok(drv.read_phys(address, mode, size)?)
        }
    }
}
