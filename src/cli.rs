use clap::{Args, Parser, Subcommand, ValueEnum};
use num_traits::{sign, Num};

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum AccessMode {
    Byte,
    Word,
    Dword,
    Qword,
    Buffer,
}

impl AccessMode {
    pub fn size(&self) -> Option<usize> {
        match self {
            Self::Byte => Some(1),
            Self::Word => Some(2),
            Self::Dword => Some(4),
            Self::Qword => Some(8),
            _ => None,
        }
    }
}

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

#[derive(Subcommand, Debug)]
pub enum Subcommands {
    /// Load the linpmem driver
    Insmod(InsmodCli),
}

#[derive(Args, Debug)]
pub struct InsmodCli {
    /// Path to the linpmem.ko object file
    pub kmod_path: String,
}

#[derive(Parser, Debug)]
/// Command line client for the linpmem driver.
///
/// Small tool for loading and interacting with the linpmem driver. It lets you
/// use the features of the driver in scripts and on the command line.
#[command(author, version)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Subcommands>,

    /// Address for physical read/write operations
    #[arg(short, long, value_parser=maybe_hex::<u64>, requires("mode"))]
    pub address: Option<u64>,

    /// Translate address in target process' address space (default: current process)
    #[arg(short, long, value_parser=maybe_hex::<u64>)]
    pub virt_address: Option<u64>,

    /// Size of buffer read operations
    #[arg(short, long, value_parser=maybe_hex::<u64>, required_if_eq("mode", "buffer"))]
    pub size: Option<u64>,

    /// Access mode for read and write operations
    #[arg(value_enum, short, long, rename_all = "lower", requires("address"))]
    pub mode: Option<AccessMode>,

    /// Write the hex-encoded byte sequence
    #[arg(short, long, requires("address"))]
    pub write: Option<String>,

    /// Target process for cr3 info and virtual-to-physical translations
    #[arg(short, long)]
    pub pid: Option<u32>,

    /// Query cr3 value of target process (default: current process)
    #[arg(long, default_value_t = false)]
    pub cr3: bool,
}
