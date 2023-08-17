use clap::{Parser, ValueEnum};
use num_traits::{sign, Num};
use std::error::Error;
use std::fs::File;
use std::io;

fn maybe_hex<T: Num + sign::Unsigned>(s: &str) -> Result<T, String>
where
    <T as num_traits::Num>::FromStrRadixErr: std::fmt::Display,
{
    const HEX_PREFIX: &str = "0x";
    const HEX_PREFIX_LEN: usize = HEX_PREFIX.len();

    let result = if s.to_ascii_lowercase().starts_with(HEX_PREFIX) {
        T::from_str_radix(&s[HEX_PREFIX_LEN..], 16)
    } else {
        T::from_str_radix(s, 10)
    };

    match result {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("{e}")),
    }
}

#[derive(Parser, Debug)]
/// Command line client for the pmem driver.
///
/// Small tool for interacting with the pmem driver. This program lets you use
/// the features of the pmem driver in scripts and on the command line.
#[command(author, version)]
pub struct Cli {
    /// Address for physical read/write operations
    #[arg(short, long, value_parser=maybe_hex::<u64>, requires("mode"))]
    address: Option<u64>,

    /// Translate address in target process' address space (default: current process)
    #[arg(short, long, value_parser=maybe_hex::<u64>)]
    virt_address: Option<u64>,

    /// Size of buffer read operations
    #[arg(short, long, value_parser=maybe_hex::<u64>, required_if_eq("mode", "buffer"))]
    size: Option<u64>,

    /// Access mode for read and write operations
    #[arg(
        value_enum,
        short,
        long,
        rename_all = "lower",
        requires("address")
    )]
    mode: Option<AccessMode>,

    /// Write the hex-encoded byte sequence
    #[arg(short, long, requires("address"))]
    write: Option<String>,

    /// Target process for cr3 info and virtual-to-physical translations
    #[arg(short, long)]
    pid: Option<u32>,

    /// Query cr3 value of target process (default: current process)
    #[arg(long, default_value_t = false)]
    cr3: bool,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum AccessMode {
    Byte,
    Word,
    Dword,
    Qword,
    Buffer,
}

pub struct Driver {
    handle: File,
}

impl Driver {
    fn build() -> io::Result<Self> {
        let handle = File::open("/dev/linpmem")?;

        Ok(Driver { handle })
    }
}

mod ioctl {
    use crate::AccessMode;
    use crate::Cli;
    use crate::Driver;

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

    pub fn cr3(
        drv: &Driver,
        pid: Option<u32>,
    ) -> Result<(), &'static str> {
        Err("Cr3 query is not implemented")
    }
    pub fn v_to_p(
        drv: &Driver,
        virt_address: u64,
        pid: Option<u32>,
    ) -> Result<(), &'static str> {
        Err("Virtual to physical translation is not implemented")
    }

    pub fn write_phys(
        drv: &Driver,
        address: u64,
        mode: AccessMode,
        data: Vec<u8>,
    ) -> Result<(), &'static str> {
        Err("Physical write is not implemented")
    }

    pub fn read_phys(
        drv: &Driver,
        address: u64,
        mode: AccessMode,
        size: Option<u64>,
    ) -> Result<(), &'static str> {
        Err("Physical write is not implemented")
    }
}

use ioctl::IOCtlCmd;

pub fn run(cli: &Cli) -> Result<(), Box<dyn Error>> {
    let drv = Driver::build()?;
    let cmd = IOCtlCmd::from_cli(cli)?;

    match cmd {
        IOCtlCmd::Cr3(pid) => Ok(ioctl::cr3(&drv, pid)?),
        IOCtlCmd::VtoP(virt_address, pid) => {
            Ok(ioctl::v_to_p(&drv, virt_address, pid)?)
        }
        IOCtlCmd::WritePhys(address, mode, data) => {
            Ok(ioctl::write_phys(&drv, address, mode, data)?)
        }
        IOCtlCmd::ReadPhys(address, mode, size) => {
            Ok(ioctl::read_phys(&drv, address, mode, size)?)
        }
    }
}
