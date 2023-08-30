/* SPDX-FileCopyrightText: © 2023 Valentin Obst <legal@bpfvol3.de>
 * SPDX-License-Identifier: MIT
 */
use std::error::Error;

mod cli;
pub mod insmod;
mod ioctl;
mod pte;
mod utils;

use crate::cli::Subcommands;
pub use crate::cli::{Cli, LoaderCli};
use crate::ioctl::{Driver, IOCtlCmd};

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
        IOCtlCmd::WritePhys(address, mode, data) => {
            Ok(drv.write_phys(address, mode, data)?)
        }
        IOCtlCmd::ReadPhys(address, mode, size) => {
            Ok(drv.read_phys(address, mode, size)?)
        }
        IOCtlCmd::CacheControl(pte_parts) => Ok(drv.cache_control(pte_parts)?),
    }
}
