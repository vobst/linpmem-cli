use crate::cli::InsmodCli;
use crate::utils;
use anyhow::{self, Context};
use log::{debug, error};
use nix::sys::stat;
use nix::{self, errno, kmod, unistd};
use std::error::Error;
use std::ffi::CString;
use std::fs;

#[derive(Debug)]
/// Builder object that is used to perform customized module loading
pub struct InsmodContext {
    module: fs::File,
    param: CString,
    major: u32,
}

impl InsmodContext {
    pub const DEV_PATH: &str = "/dev/linpmem";
    pub const DRV_NAME: &str = "linpmem";

    fn from_cli(cli: &InsmodCli) -> anyhow::Result<Self> {
        Self::build(
            cli.kmod_path
                .as_ref()
                .context("Please specify a path to the driver object")?,
        )
    }

    /// Create an InsmodContext instance that can be used to load the module
    pub fn build(module: &str) -> anyhow::Result<Self> {
        let major = Self::find_unused_major()
            .context("Failed to find an unused major number")?;
        Ok(InsmodContext {
            module: fs::File::open(module)
                .context(format!("Failed to open {}", module))?,
            param: Self::build_param(major)?,
            major,
        })
    }

    // Todo: iterate /dev and find a chrdev major that is unused
    fn find_unused_major() -> anyhow::Result<u32> {
        Ok(42)
    }

    fn build_param(major: u32) -> anyhow::Result<CString> {
        let param = format!("major={}", major);

        debug!("Module load parameters: {}", param);

        Ok(CString::new(param)?)
    }

    /// Load the module
    pub fn load(self) -> Result<Self, nix::errno::Errno> {
        if let Err(err) = kmod::finit_module(
            &self.module,
            self.param.as_c_str(),
            kmod::ModuleInitFlags::empty(),
        ) {
            error!("Module was rejected by kernel");
            return Err(err);
        };

        Ok(self)
    }

    /// Remove the module and delete the device special file
    pub fn unload() -> Result<(), nix::errno::Errno> {
        if let Err(err) = kmod::delete_module(
            &CString::new(Self::DRV_NAME)
                .expect("BUG: DRV_NAME cannot be converted to C string"),
            kmod::DeleteModuleFlags::O_NONBLOCK,
        ) {
            error!("Failed to unload module");
            return Err(err);
        };

        if let Err(err) = unistd::unlink(Self::DEV_PATH) {
            error!("Failed to remove {}", Self::DEV_PATH);
            return Err(err);
        }

        Ok(())
    }

    /// Create the device special file
    ///
    /// Returns no error if the file already exists, i.e., it silently re-uses
    /// an existing file.
    pub fn mknod(self) -> Result<Self, nix::errno::Errno> {
        if let Err(err) = stat::mknod(
            Self::DEV_PATH,
            stat::SFlag::S_IFCHR,
            stat::Mode::S_IRUSR | stat::Mode::S_IRGRP | stat::Mode::S_IROTH,
            stat::makedev(self.major as u64, 0),
        ) {
            if err != errno::Errno::EEXIST {
                error!(
                    "Failed to create device special file {}",
                    Self::DEV_PATH
                );
                return Err(err);
            }
        };

        Ok(self)
    }
}

pub mod ffi {
    //! Public C/C++ API for loading and unloading of the driver.

    use super::InsmodContext;
    use std::ffi::{c_char, c_int, CStr};

    #[no_mangle]
    /// pmem_load - load the linpmem driver
    /// @path: pointer to a string with the path to the driver object
    ///
    /// This must be called to load the linpmem driver prior to using it.
    ///
    /// Returns zero on success, or -EXXX on failure
    pub extern "C" fn pmem_load(path: *const c_char) -> c_int {
        if path.is_null() {
            return -1;
        }

        let path = unsafe { CStr::from_ptr(path) }.to_str();
        let Ok(path) = path else { return -1; };

        let ctx = InsmodContext::build(path);
        let Ok(ctx) = ctx else {
            return -1;
        };

        let ctx = ctx.load();
        let Ok(ctx) = ctx else {
            return ctx.unwrap_err() as c_int;
        };

        let ctx = ctx.mknod();
        if let Err(err) = ctx {
            return err as c_int;
        };

        0
    }

    #[no_mangle]
    /// pmem_unload - unload the linpmem driver
    ///
    /// This can be called to unload the linpmem driver after using it.
    ///
    /// Returns zero on success, or -EXXX on failure
    pub extern "C" fn pmem_unload() -> c_int {
        match InsmodContext::unload() {
            Err(errno) => return errno as c_int,
            Ok(()) => 0,
        }
    }
}

pub fn run(cli: &InsmodCli) -> Result<(), Box<dyn Error>> {
    utils::check_root()?;

    if cli.rm {
        return Ok(InsmodContext::unload()?);
    }

    InsmodContext::from_cli(cli)?.load()?.mknod()?;

    Ok(())
}
