use crate::cli::{AccessMode, Cli};
use crate::insmod::InsmodContext;
use anyhow::{bail, Context};
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::os::fd::AsRawFd;

mod ffi;

#[derive(Debug)]
pub enum IOCtlCmd {
    VtoP(u64, Option<u32>),
    Cr3(Option<u32>),
    ReadPhys(u64, AccessMode, Option<u64>),
}

impl IOCtlCmd {
    pub fn from_cli(cli: &Cli) -> anyhow::Result<Self> {
        if cli.cr3 {
            return Ok(Self::Cr3(cli.pid));
        }
        if let Some(virt_address) = cli.virt_address {
            return Ok(Self::VtoP(virt_address, cli.pid));
        }
        if let Some(address) = cli.address {
            return Ok(Self::ReadPhys(address, cli.mode.unwrap(), cli.size));
        }

        bail!("Invalid combination of arguments")
    }
}

pub struct Driver {
    handle: File,
}

impl Driver {
    pub fn build() -> anyhow::Result<Self> {
        let handle = File::open(InsmodContext::DEV_PATH)
            .context("Cannot open device file. Does it exist and do I have the permission to open it?")?;

        Ok(Self { handle })
    }

    pub fn cr3(&self, pid: Option<u32>) -> Result<(), Box<dyn Error>> {
        let cr3_pa = ffi::cr3(self.handle.as_raw_fd(), pid)?;

        println!("0x{:016x}", cr3_pa);

        Ok(())
    }

    pub fn v_to_p(
        &self,
        virt_address: u64,
        pid: Option<u32>,
    ) -> Result<(), Box<dyn Error>> {
        let phys_address =
            ffi::v_to_p(self.handle.as_raw_fd(), virt_address, pid)?;

        println!("0x{:016x}", phys_address);

        Ok(())
    }

    pub fn read_phys(
        &self,
        address: u64,
        mode: AccessMode,
        size: Option<u64>,
    ) -> Result<(), Box<dyn Error>> {
        let mem = ffi::read_phys(self.handle.as_raw_fd(), address, mode, size)?;

        io::stdout().write_all(mem.as_slice())?;

        Ok(())
    }
}
