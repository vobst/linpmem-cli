use crate::cli::InsmodCli;
use crate::utils;
use nix::sys::stat;
use nix::{self, errno, kmod, unistd};
use std::error::Error;
use std::ffi::CString;
use std::fs;
use log::{debug, error};

mod loader;

mod kallsyms {
    use anyhow::{anyhow, bail, Context};
    use std::fs;
    use std::io::{self, BufRead, Read, Seek, Write};

    /// Do a lookup of a name in /proc/kallsyms
    pub fn lookup(name: &str) -> anyhow::Result<u64> {
        let mut kptr_restrict = fs::File::options()
            .read(true)
            .write(true)
            .open("/proc/sys/kernel/kptr_restrict")
            .context("Failed to open sysctl kptr_restrict")?;
        let mut old = [0; 1];
        kptr_restrict.read_exact(&mut old)?;
        if old[0] == b'2' {
            kptr_restrict.seek(io::SeekFrom::Start(0))?;
            kptr_restrict.write_all(&[b'1'])?;
        }

        let file = fs::File::open("/proc/kallsyms")?;
        let matches: Vec<String> = io::BufReader::new(file)
            .lines()
            .filter(|line| match line {
                Ok(line) => line.ends_with(format!(" {}", name).as_str()),
                Err(err) => {
                    println!("{}", err);
                    false
                }
            })
            .map(|line| line.unwrap())
            .collect();

        if old[0] == b'2' {
            kptr_restrict.seek(io::SeekFrom::Start(0))?;
            kptr_restrict.write_all(&old)?;
        }

        if matches.len() != 1 {
            bail!(
                "Found {} matches for {} in kallsyms, expected 1",
                matches.len(),
                name
            );
        }

        let address: u64 =
            u64::from_str_radix(matches[0].split(' ').next().unwrap(), 16)?;

        if address == 0 {
            Err(anyhow!("Address of {} in kallsyms is zero", name))
        } else {
            Ok(address)
        }
    }
}

#[derive(Debug)]
/// Builder object that is used to perform customized module loading
pub struct InsmodContext {
    adjust: bool,
    module: fs::File,
    adjusted_module: Option<Vec<u8>>, // set iff adjust == true
    valid_module: Option<Vec<u8>>,    // set iff adjust == true
    param: CString,
    major: u32,
}

impl InsmodContext {
    pub const DEV_PATH: &str = "/dev/linpmem";
    pub const DRV_NAME: &str = "linpmem";

    fn from_cli(cli: &InsmodCli) -> Result<Self, Box<dyn Error>> {
        Self::build(
            cli.adjust,
            cli.kmod_path
                .as_ref()
                .ok_or("Please specify a path to the driver object")?,
            cli.valid_driver.as_ref(),
        )
    }

    /// Create an InsmodContext instance that can be used to load the module
    pub fn build(
        adjust: bool,
        module: &str,
        valid_module: Option<&String>,
    ) -> Result<Self, Box<dyn Error>> {
        let valid_module = if adjust {
            let (valid_module, name) = match valid_module {
                Some(path) => (fs::File::open(path)?, path.to_owned()),
                None => loader::find_valid_module()?,
            };
            Some(loader::mod_to_vec_decompress(valid_module, name)?)
        } else {
            None
        };
        let major = Self::find_unused_major()?;
        Ok(InsmodContext {
            adjust,
            module: fs::File::open(module)?,
            adjusted_module: None,
            valid_module,
            param: Self::build_param(major)?,
            major,
        })
    }

    // Todo: iterate /dev and find a chrdev major that is unused
    fn find_unused_major() -> Result<u32, Box<dyn Error>> {
        Ok(42)
    }

    fn build_param(major: u32) -> Result<CString, Box<dyn Error>> {
        let mut param = format!("major={}", major);
        let res = kallsyms::lookup("kallsyms_lookup_name");

        if let Ok(kallsyms_lookup_name) = res {
            param.push_str(format!(
                " kallsyms_lookup_name={}",
                kallsyms_lookup_name
            ).as_str());
        } else {
            debug!(
                "User space kallsyms -> kallsyms_lookup_name failed: {}",
                res.unwrap_err()
            );
        }

        debug!("module load parameters: {}", param);

        Ok(CString::new(param)?)
    }

    /// Adjusts the module to the running kernel
    pub fn adjust(self) -> Result<Self, Box<dyn Error>> {
        if !self.adjust {
            return Ok(self);
        }
        Ok(Self {
            adjusted_module: Some(loader::adjust_module(
                &self.module,
                self.valid_module
                    .as_ref()
                    .ok_or("Internal error: valid_module was None")?,
            )?),
            ..self
        })
    }

    /// Load the module
    ///
    /// Either loads the unmodified module from directly a file or loads the
    /// adjusted module from a buffer.
    pub fn load(self) -> Result<Self, nix::errno::Errno> {
        let res;

        if self.adjust {
            res = kmod::init_module(
                self.adjusted_module
                    .as_ref()
                    .expect("BUG: adjusted_module was None"),
                self.param.as_c_str(),
            );
        } else {
            res = kmod::finit_module(
                &self.module,
                self.param.as_c_str(),
                kmod::ModuleInitFlags::empty(),
            );
        }

        match res {
            Ok(()) => Ok(self),
            Err(err) => {
                error!("Module was rejected by kernel: {}", err);
                Err(err)
            },
        }
    }

    /// Remove the module and delete the device special file
    pub fn unload() -> Result<(), nix::errno::Errno> {
        kmod::delete_module(
            &CString::new(Self::DRV_NAME)
                .expect("BUG: DRV_NAME cannot be converted to C string"),
            kmod::DeleteModuleFlags::O_NONBLOCK,
        )?;
        unistd::unlink(Self::DEV_PATH)?;
        Ok(())
    }

    /// Create the device special file
    ///
    /// Returns no error if the file already exists
    pub fn mknod(self) -> Result<Self, nix::errno::Errno> {
        if let Err(err) = stat::mknod(
            Self::DEV_PATH,
            stat::SFlag::S_IFCHR,
            stat::Mode::S_IRUSR | stat::Mode::S_IRGRP | stat::Mode::S_IROTH,
            stat::makedev(self.major as u64, 0),
        ) {
            if err != errno::Errno::EEXIST {
                return Err(err);
            }
        };

        Ok(self)
    }
}

pub mod ffi {
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

        let ctx = InsmodContext::build(false, path, None);
        let Ok(ctx) = ctx else {
            return -1;
        };

        let ctx = ctx.adjust();
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

    InsmodContext::from_cli(cli)?.adjust()?.load()?.mknod()?;

    Ok(())
}
