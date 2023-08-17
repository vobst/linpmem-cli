use std::error::Error;

mod cli;
mod ioctl;

pub use crate::cli::Cli;
use crate::ioctl::{IOCtlCmd, Driver};

pub fn run(cli: &Cli) -> Result<(), Box<dyn Error>> {
    let cmd = IOCtlCmd::from_cli(cli)?;
    let drv = Driver::build()?;

    match cmd {
        IOCtlCmd::Cr3(pid) => Ok(drv.cr3(pid)?),
        IOCtlCmd::VtoP(virt_address, pid) => {
            Ok(drv.v_to_p(virt_address, pid)?)
        }
        IOCtlCmd::WritePhys(address, mode, data) => {
            Ok(drv.write_phys(address, mode, data)?)
        }
        IOCtlCmd::ReadPhys(address, mode, size) => {
            Ok(drv.read_phys(address, mode, size)?)
        }
    }
}
