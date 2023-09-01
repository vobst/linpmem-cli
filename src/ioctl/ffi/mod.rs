//! Safe bindings for all IOCTLs supported by the driver

/// cbindgen:ignore
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
ioctl_readwrite!(unsafe_v_to_p, b'a', b'b', bindings::LINPMEM_VTOP_INFO);
ioctl_readwrite!(unsafe_cr3, b'a', b'c', bindings::LINPMEM_CR3_INFO);

impl From<AccessMode> for u8 {
    fn from(value: AccessMode) -> u8 {
        match value {
            AccessMode::Byte => {
                bindings::_PHYS_ACCESS_MODE_PHYS_BYTE_READ as u8
            }
            AccessMode::Word => {
                bindings::_PHYS_ACCESS_MODE_PHYS_WORD_READ as u8
            }
            AccessMode::Dword => {
                bindings::_PHYS_ACCESS_MODE_PHYS_DWORD_READ as u8
            }
            AccessMode::Qword => {
                bindings::_PHYS_ACCESS_MODE_PHYS_QWORD_READ as u8
            }
            AccessMode::Buffer => {
                bindings::_PHYS_ACCESS_MODE_PHYS_BUFFER_READ as u8
            }
        }
    }
}

pub fn read_phys(
    fd: fd::RawFd,
    address: u64,
    mode: AccessMode,
    size: Option<u64>,
) -> Result<Vec<u8>, nix::errno::Errno> {
    // cannot panic on 64 bit as usize == u64
    let mut mem: Vec<u8> =
        Vec::with_capacity(size.unwrap_or(0).try_into().unwrap());
    let mut data_transfer = bindings::LINPMEM_DATA_TRANSFER {
        phys_address: address,
        out_value: 0,
        readbuffer: match mode {
            AccessMode::Buffer => mem.as_mut_ptr() as *mut std::ffi::c_void,
            _ => ptr::null_mut(),
        },
        readbuffer_size: size.unwrap_or(0),
        access_type: u8::from(mode),
        write_access: 0,
        reserved1: 0,
        reserved2: 0,
    };

    let _result = unsafe { unsafe_read_write_pyhs(fd, &mut data_transfer) }?;

    match mode {
        AccessMode::Buffer => {
            // Vector does not know that kernel gave it some data. Still
            // thinks it is empty. Luckily the kernel told us how much it
            // wrote.
            unsafe {
                // cannot panic on 64 bit
                mem.set_len(data_transfer.readbuffer_size.try_into().unwrap())
            };
            Ok(mem)
        }
        _ => Ok(
            data_transfer.out_value.to_le_bytes()[..mode.size().unwrap()]
                .into(),
        ),
    }
}

pub fn v_to_p(
    fd: fd::RawFd,
    virt_address: u64,
    pid: Option<u32>,
) -> Result<u64, nix::errno::Errno> {
    let mut data_transfer = bindings::LINPMEM_VTOP_INFO {
        virt_address,
        associated_cr3: match pid {
            Some(pid) => cr3(fd, Some(pid))?,
            None => 0,
        },
        phys_address: 0,
        ppte: ptr::null_mut(),
    };

    let _result = unsafe { unsafe_v_to_p(fd, &mut data_transfer) }?;

    Ok(data_transfer.phys_address)
}

pub fn cr3(fd: fd::RawFd, pid: Option<u32>) -> Result<u64, nix::errno::Errno> {
    let mut data_transfer = bindings::LINPMEM_CR3_INFO {
        target_process: pid.unwrap_or(0) as u64,
        result_cr3: 0,
    };

    let _result = unsafe { unsafe_cr3(fd, &mut data_transfer) }?;

    Ok(data_transfer.result_cr3)
}
