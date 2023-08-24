use crate::cli::InsmodCli;
use crate::utils;
use nix::sys::stat;
use nix::{self, kmod};
use std::error::Error;
use std::ffi::CString;
use std::fs;

pub struct LoadContext {
    object: fs::File,
    param: CString,
    major: u32,
}

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

impl LoadContext {
    pub const DEV_PATH: &str = "/dev/linpmem";

    fn build(cli: &InsmodCli) -> Result<Self, Box<dyn Error>> {
        let major = Self::find_unused_major()?;
        Ok(LoadContext {
            object: fs::File::open(&cli.kmod_path)?,
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

    fn mknod(&self) -> Result<(), nix::errno::Errno> {
        stat::mknod(
            Self::DEV_PATH,
            stat::SFlag::S_IFCHR,
            stat::Mode::S_IRUSR | stat::Mode::S_IRGRP | stat::Mode::S_IROTH,
            stat::makedev(self.major as u64, 0),
        )
    }
}

pub fn run(cli: &InsmodCli) -> Result<(), Box<dyn Error>> {
    utils::check_root()?;

    let ctx = LoadContext::build(cli)?;

    ctx.load()?;

    match ctx.mknod() {
        Ok(()) => Ok(()),
        Err(e) => match e {
            nix::errno::Errno::EEXIST => {
                println!("Reusing existing device file {}", LoadContext::DEV_PATH);
                Ok(())
            },
            _ => Err(Box::new(e)),
        }
    }
}
