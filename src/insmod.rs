use crate::cli::InsmodCli;
use nix::{self, kmod};
use std::error::Error;
use std::ffi::CString;
use std::fs;

struct LoadContext {
    object: fs::File,
    param: CString,
}

mod kallsyms {
    use std::error::Error;
    use std::fs;
    use std::io::{self, BufRead};

    struct Env {
        restrict: bool,
        procfs: bool,
        kprobes: bool,
    }

    pub fn lookup(name: &str) -> Result<u64, Box<dyn Error>> {
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

        if matches.len() != 1 {
            return Err(format!(
                "Found {} matches for {} in kallsyms",
                matches.len(),
                name
            )
            .into());
        }

        let address: u64 =
            u64::from_str_radix(matches[0].split(" ").next().unwrap(), 16)?;

        if address == 0 {
            Err("kallsyms_restrict == 2".into())
        } else {
            Ok(address)
        }
    }
}

impl LoadContext {
    fn build(cli: &InsmodCli) -> Result<Self, Box<dyn Error>> {
        Ok(LoadContext {
            object: fs::File::open(&cli.kmod_path)?,
            param: Self::build_param()?,
        })
    }

    fn build_param() -> Result<CString, Box<dyn Error>> {
        let res = kallsyms::lookup("kallsyms_lookup_name");

        if let Ok(kallsyms_lookup_name) = res {
            return Ok(CString::new(format!(
                "kallsyms_lookup_name={}",
                kallsyms_lookup_name
            ))?);
        }
        println!("{}", res.unwrap_err());

        Err("Unable to obtain the address of kallsyms_lookup_name".into())
    }

    fn load(&self) -> Result<(), nix::errno::Errno> {
        kmod::finit_module(
            &self.object,
            self.param.as_c_str(),
            kmod::ModuleInitFlags::empty(),
        )?;

        Ok(())
    }
}

pub fn run(cli: &InsmodCli) -> Result<(), Box<dyn Error>> {
    let ctx = LoadContext::build(&cli)?;

    ctx.load()?;

    Ok(())
}
