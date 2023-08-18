use crate::cli::AccessMode;
use crate::cli::Cli;
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::os::fd::AsRawFd;

mod ffi {
    #[allow(non_upper_case_globals, unused, non_camel_case_types)]
    mod bindings;

    use crate::cli::AccessMode;
    use nix::ioctl_readwrite;
    use std::os::fd;
    use std::ptr;

    ioctl_readwrite!(
        unsafe_read_write_pyhs,
        b'a',
        b'a',
        bindings::LINPMEM_DATA_TRANSFER
    );
    ioctl_readwrite!(
        unsafe_v_to_p,
        b'a',
        b'b',
        bindings::LINPMEM_VTOP_INFO
    );
    ioctl_readwrite!(
        unsafe_cr3,
        b'a',
        b'c',
        bindings::LINPMEM_CR3_INFO
    );

    pub fn read_phys(
        fd: fd::RawFd,
        address: u64,
        mode: AccessMode,
        size: Option<u64>,
    ) -> Result<Vec<u8>, nix::errno::Errno> {
        // cannot panic on 64 bit as usize == u64
        let mut out_vec: Vec<u8> =
            Vec::with_capacity(size.unwrap_or(0).try_into().unwrap());
        let mut data_transfer = bindings::LINPMEM_DATA_TRANSFER {
            phys_address: address,
            out_value: 0,
            readbuffer: match mode {
                AccessMode::Buffer => {
                    out_vec.as_mut_ptr() as *mut std::ffi::c_void
                }
                _ => ptr::null_mut(),
            },
            readbuffer_size: size.unwrap_or(0),
            access_type: match mode {
                AccessMode::Byte => {
                    bindings::_PHYS_ACCESS_MODE_PHYS_BYTE_READ as u8
                }
                AccessMode::Word => {
                    bindings::_PHYS_ACCESS_MODE_PHYS_WORD_READ as u8
                }
                AccessMode::Dword => {
                    bindings::_PHYS_ACCESS_MODE_PHYS_WORD_READ as u8
                }
                AccessMode::Qword => {
                    bindings::_PHYS_ACCESS_MODE_PHYS_QWORD_READ as u8
                }
                AccessMode::Buffer => {
                    bindings::_PHYS_ACCESS_MODE_PHYS_BUFFER_READ as u8
                }
            },
            write_access: 0,
            reserved1: 0,
            reserved2: 0,
        };

        let _result =
            unsafe { unsafe_read_write_pyhs(fd, &mut data_transfer) }?;

        match mode {
            AccessMode::Buffer => {
                // Vector does not know that kernel gave it some data. Still
                // thinks it is empty. Luckily the kernel told us how much it
                // wrote.
                unsafe {
                    out_vec.set_len(
                        data_transfer
                            .readbuffer_size
                            .try_into()
                            .unwrap(), // cannot panic on 64 bit
                    )
                };
                Ok(out_vec)
            }
            _ => Ok(data_transfer.out_value.to_le_bytes()
                [..mode.size().unwrap()]
                .into()),
        }
    }
}

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
    ) -> Result<(), Box<dyn Error>> {
        let mem = ffi::read_phys(
            self.handle.as_raw_fd(),
            address,
            mode,
            size,
        )?;

        io::stdout().write(mem.as_slice())?;

        Ok(())
    }
}