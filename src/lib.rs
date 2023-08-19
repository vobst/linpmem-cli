use std::error::Error;

mod cli;
mod ioctl;
mod insmod;
mod utils;

pub use crate::cli::Cli;
use crate::cli::Subcommands;
use crate::ioctl::{Driver, IOCtlCmd};

pub fn run(cli: &Cli) -> Result<(), Box<dyn Error>> {
    utils::check_root()?;

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
    }
}
