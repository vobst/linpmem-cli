use clap::{Parser, ValueEnum};
use num_traits::{sign, Num};

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
struct Cli {
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
    #[arg(value_enum, short, long, rename_all = "lower", requires("address"))]
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

#[derive(ValueEnum, Clone, Debug)]
enum AccessMode {
    Byte,
    Word,
    Dword,
    Qword,
    Buffer,
}

fn main() {
    let args = Cli::parse();

    if let Some(pid) = args.pid {
        println!("{}", pid);
    }
}