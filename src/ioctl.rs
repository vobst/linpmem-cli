use crate::cli::AccessMode;
use crate::cli::Cli;
use crate::insmod::InsmodContext;
use crate::pte::Pte;
use crate::pte::PteParts;
use anyhow::{bail, Context};
use log::debug;
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::os::fd::AsRawFd;

mod ffi;

#[derive(Debug)]
pub enum IOCtlCmd<'a> {
    VtoP(u64, Option<u32>),
    Cr3(Option<u32>),
    ReadPhys(u64, AccessMode, Option<u64>),
    WritePhys(u64, AccessMode, Vec<u8>),
    CacheControl(&'a Vec<PteParts>),
}

impl<'a> IOCtlCmd<'a> {
    pub fn from_cli(cli: &'a Cli) -> anyhow::Result<Self> {
        if let Some(pte_parts) = cli.pte_parts.as_ref() {
            debug!("Running cache control with pte parts: {:?}", pte_parts);
            return Ok(IOCtlCmd::CacheControl(pte_parts));
        }
        if cli.cr3 {
            return Ok(IOCtlCmd::Cr3(cli.pid));
        }
        if let Some(virt_address) = cli.virt_address {
            return Ok(Self::VtoP(virt_address, cli.pid));
        }
        if let Some(_hex_string) = cli.write.clone() {
            return Ok(Self::WritePhys(
                cli.address.unwrap(), // ok as Clap enforces them if write
                cli.mode.unwrap(),    // is given
                Vec::new(),
            ));
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

    pub fn write_phys(
        &self,
        _address: u64,
        _mode: AccessMode,
        _data: Vec<u8>,
    ) -> Result<(), &'static str> {
        Err("Writing of physical memory is not implemented")
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

    pub fn cache_control(
        &self,
        pte_parts: &Vec<PteParts>,
    ) -> anyhow::Result<()> {
        if pte_parts.is_empty() {
            debug!("Querying current template PTE");
            let template_pte = Pte::try_from(ffi::cache_control_get(
                self.handle.as_raw_fd(),
            )?)?;
            println!("{}", template_pte)
        } else {
            let pte = Pte::try_from(pte_parts)?;
            debug!("Setting template PTE to {}", pte);
            ffi::cache_control_set(self.handle.as_raw_fd(), pte)?;
        }
        Ok(())
    }
}
