use crate::cli::InsmodCli;
use crate::utils;
use nix::sys::stat;
use nix::{self, kmod, unistd};
use std::error::Error;
use std::ffi::CString;
use std::fs;

mod kallsyms {
    use std::error::Error;
    use std::fs;
    use std::io::{self, BufRead, Read, Seek, Write};

    pub fn lookup(name: &str) -> Result<u64, Box<dyn Error>> {
        let mut kptr_restrict = fs::File::options()
            .read(true)
            .write(true)
            .open("/proc/sys/kernel/kptr_restrict")?;
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
            return Err(format!(
                "Found {} matches for {} in kallsyms",
                matches.len(),
                name
            )
            .into());
        }

        let address: u64 =
            u64::from_str_radix(matches[0].split(' ').next().unwrap(), 16)?;

        if address == 0 {
            Err(format!("Address of {} in kallsyms is zero", name).into())
        } else {
            Ok(address)
        }
    }
}

pub struct LoadContext {
    object: fs::File,
    param: CString,
    major: u32,
}

impl LoadContext {
    pub const DEV_PATH: &str = "/dev/linpmem";
    pub const DRV_NAME: &str = "linpmem";

    fn from_cli(cli: &InsmodCli) -> Result<Self, Box<dyn Error>> {
        let major = Self::find_unused_major()?;
        Ok(LoadContext {
            object: fs::File::open(
                cli.kmod_path
                    .as_ref()
                    .ok_or("Please specify a path to the driver object")?,
            )?,
            param: Self::build_param(major)?,
            major,
        })
    }

    fn build(path: &str) -> Result<Self, Box<dyn Error>> {
        let major = Self::find_unused_major()?;
        Ok(LoadContext {
            object: fs::File::open(path)?,
            param: Self::build_param(major)?,
            major,
        })
    }

    // Todo: iterate /dev and find a chrdev major that is unused
    fn find_unused_major() -> Result<u32, Box<dyn Error>> {
        Ok(42)
    }

    fn build_param(major: u32) -> Result<CString, Box<dyn Error>> {
        let res = kallsyms::lookup("kallsyms_lookup_name");

        if let Ok(kallsyms_lookup_name) = res {
            return Ok(CString::new(format!(
                "kallsyms_lookup_name={} major={}",
                kallsyms_lookup_name, major
            ))?);
        }
        println!(
            "User space kallsyms -> kallsyms_lookup_name failed: {}",
            res.unwrap_err()
        );

        Err("Unable to build module parameters".into())
    }

    fn load(&self) -> Result<(), nix::errno::Errno> {
        kmod::finit_module(
            &self.object,
            self.param.as_c_str(),
            kmod::ModuleInitFlags::empty(),
        )?;

        Ok(())
    }

    fn unload() -> Result<(), nix::errno::Errno> {
        kmod::delete_module(
            &CString::new(Self::DRV_NAME).expect("BUG"),
            kmod::DeleteModuleFlags::O_NONBLOCK,
        )?;
        unistd::unlink(Self::DEV_PATH)?;
        Ok(())
    }

    fn mknod(&self) -> Result<(), nix::errno::Errno> {
        stat::mknod(
            Self::DEV_PATH,
            stat::SFlag::S_IFCHR,
            stat::Mode::S_IRUSR | stat::Mode::S_IRGRP | stat::Mode::S_IROTH,
            stat::makedev(self.major as u64, 0),
        )
    }
}

pub mod ffi {
    use super::LoadContext;
    use nix::errno::Errno;
    use std::ffi;

    #[no_mangle]
    /// pmem_load - load the linpmem driver
    /// @path: pointer to a string with the path to the driver object
    ///
    /// This must be called to load the linpmem driver prior to using it.
    ///
    /// Returns zero on success, or -EXXX on failure
    pub extern "C" fn pmem_load(path: *const ffi::c_char) -> i32 {
        if path.is_null() {
            return -1;
        }

        let path = unsafe { ffi::CStr::from_ptr(path) }.to_str();
        let Ok(path) = path else { return -1; };
        let ctx = LoadContext::build(path);
        let Ok(ctx) = ctx else { return -1; };

        let mut res = ctx.load();
        if let Err(errno) = res {
            return errno as i32;
        }

        res = ctx.mknod();
        if let Err(errno) = res {
            if errno != Errno::EEXIST {
                return errno as i32;
            }
        }

        0
    }

    #[no_mangle]
    /// pmem_unload - unload the linpmem driver
    ///
    /// This can be called to unload the linpmem driver after using it.
    ///
    /// Returns zero on success, or -EXXX on failure
    pub extern "C" fn pmem_unload() -> i32 {
        match LoadContext::unload() {
            Err(errno) => return errno as i32,
            Ok(()) => 0,
        }
    }
}

pub fn run(cli: &InsmodCli) -> Result<(), Box<dyn Error>> {
    utils::check_root()?;

    if cli.rm {
        return Ok(LoadContext::unload()?);
    }

    let ctx = LoadContext::from_cli(cli)?;

    ctx.load()?;

    match ctx.mknod() {
        Ok(()) => Ok(()),
        Err(e) => match e {
            nix::errno::Errno::EEXIST => {
                println!(
                    "Reusing existing device file {}",
                    LoadContext::DEV_PATH
                );
                Ok(())
            }
            _ => Err(Box::new(e)),
        },
    }
}
