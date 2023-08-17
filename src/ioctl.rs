use crate::cli::AccessMode;
use crate::cli::Cli;
use std::fs::File;
use std::io;

#[derive(Debug)]
pub enum IOCtlCmd {
    VtoP(u64, Option<u32>),
    Cr3(Option<u32>),
    ReadPhys(u64, AccessMode, Option<u64>),
    WritePhys(u64, AccessMode, Vec<u8>),
}

impl IOCtlCmd {
    pub fn from_cli(cli: &Cli) -> Result<IOCtlCmd, &'static str> {
        if cli.cr3 {
            return Ok(IOCtlCmd::Cr3(cli.pid));
        }
        if let Some(virt_address) = cli.virt_address {
            return Ok(Self::VtoP(virt_address, cli.pid));
        }
        if let Some(hex_string) = cli.write.clone() {
            return Ok(Self::WritePhys(
                cli.address.unwrap(), // ok as Clap enforces them if write
                cli.mode.unwrap(),    // is given
                Vec::new(),
            ));
        }
        if let Some(address) = cli.address {
            return Ok(Self::ReadPhys(
                address,
                cli.mode.unwrap(),
                cli.size,
            ));
        }

        Err("Invalid combination of arguments")
    }
}

pub struct Driver {
    handle: File,
}

impl Driver {
    pub fn build() -> io::Result<Self> {
        let handle = File::open("/dev/linpmem")?;

        Ok(Self { handle })
    }

    pub fn cr3(&self, pid: Option<u32>) -> Result<(), &'static str> {
        Err("Cr3 query is not implemented")
    }

    pub fn v_to_p(
        &self,
        virt_address: u64,
        pid: Option<u32>,
    ) -> Result<(), &'static str> {
        Err("Virtual to physical translation is not implemented")
    }

    pub fn write_phys(
        &self,
        address: u64,
        mode: AccessMode,
        data: Vec<u8>,
    ) -> Result<(), &'static str> {
        Err("Physical write is not implemented")
    }

    pub fn read_phys(
        &self,
        address: u64,
        mode: AccessMode,
        size: Option<u64>,
    ) -> Result<(), &'static str> {
        Err("Physical write is not implemented")
    }
}
