use anyhow::{bail, Context};
use goblin;
use log::debug;
use nix::sys::utsname;
use ruzstd;
use std::{fmt::Display, fs, io::Read, path};

#[derive(Debug)]
enum ModuleCompression {
    Zstd,
    Xz,
    Gz,
    No,
}

impl TryFrom<&str> for ModuleCompression {
    type Error = anyhow::Error;

    fn try_from(name: &str) -> anyhow::Result<Self> {
        if name.ends_with(".ko") {
            return Ok(Self::No);
        } else if name.ends_with(".ko.zst") {
            return Ok(Self::Zstd);
        } else if name.ends_with(".ko.xz") {
            return Ok(Self::Xz);
        } else if name.ends_with(".ko.gz") {
            return Ok(Self::Gz);
        } else {
            bail!("Module {} uses unkown compression", name)
        }
    }
}

/// Searches for a module that is likely to be compatible to the running kernel
pub fn find_valid_module() -> anyhow::Result<(fs::File, String)> {
    let search_prefix = path::PathBuf::from(format!(
        "/usr/lib/modules/{}/",
        utsname::uname()?
            .release()
            .to_str()
            .context("Kernel release is not valid unicode")?
    ));
    debug!(
        "Searching modules in {}",
        search_prefix
            .to_str()
            .expect("Internal error: search path is not valid unicode")
    );
    let mut stack: Vec<path::PathBuf> = Vec::new();
    stack.push(search_prefix);
    while !stack.is_empty() {
        let dir = stack.pop().unwrap();
        for entry in dir.read_dir()? {
            let entry = entry?;
            let name = entry.file_name();
            let name =
                name.to_str().context("File name is not valid unicode")?;
            let path = entry.path();
            let file_type =
                entry.file_type().context("Can not get file type")?;

            if file_type.is_dir() {
                stack.push(path);
                continue;
            }

            if file_type.is_symlink() {
                continue;
            }

            if file_type.is_file() {
                let compression = ModuleCompression::try_from(name);
                if let Ok(compression) = compression {
                    debug!(
                        "Found valid module {} with compression {:?}",
                        name, compression
                    );
                    return Ok((fs::File::open(path)?, name.to_owned()));
                }
            }
        }
    }
    bail!("Unable to find valid module")
}

/// Loads a, potentially compressed, module from a file into a buffer
pub fn mod_to_vec_decompress<T: AsRef<str> + Display>(
    mut module: fs::File,
    name: T,
) -> anyhow::Result<Vec<u8>> {
    let mut vec = Vec::new();

    match ModuleCompression::try_from(name.as_ref())? {
        ModuleCompression::No => module
            .read_to_end(&mut vec)
            .context("Failed to read uncompressed module")?,
        ModuleCompression::Zstd => ruzstd::StreamingDecoder::new(module)?
            .read_to_end(&mut vec)
            .context("Failed to read compressed module")?,
        compression => {
            bail!("Decompression of {:?} is not implemented", compression)
        }
    };

    debug!("Decompressed {} starts with {:x?}", name, &vec[0..8]);

    Ok(vec)
}

/// Values gathered from the environment that are needed to adjust the module
struct AdjustContext {
    vermagic: String,
}

impl AdjustContext {
    fn build(valid_module: &Vec<u8>) -> anyhow::Result<Self> {
        let goblin::Object::Elf(elf) = goblin::Object::parse(valid_module)
            .context("Failed to parse valid module")? else {
                bail!("Valid module is not an ELF file");
            };
        let vermagic = String::from(
            kmod_parser::Modinfo::build(valid_module, &elf)?
                .get_value("vermagic")?,
        );
        debug!("Valid module has vermagic {}", &vermagic);

        Ok(Self { vermagic })
    }
}

/// Performs various adjustments to a module in order to make it loadable for
/// the current kernel.
pub fn adjust_module(
    module: &fs::File,
    valid_module: &Vec<u8>,
) -> anyhow::Result<Vec<u8>> {
    let ctx = AdjustContext::build(valid_module)?;

    let mut adjusted_module = Vec::new();
    module
        .try_clone()?
        .read_to_end(&mut adjusted_module)
        .context("Failed to read the module to adjust")?;

    Ok(adjusted_module)
}

mod kmod_parser {
    use anyhow::{bail, Context};
    use goblin::{elf, strtab};

    pub struct Modinfo<'a> {
        strtab: strtab::Strtab<'a>,
    }

    impl<'a> Modinfo<'a> {
        pub fn build(raw: &'a Vec<u8>, elf: &elf::Elf) -> anyhow::Result<Self> {
            let sh_modinfo = find_sh_by_name(elf, ".modinfo")?;
            let modinfo = strtab::Strtab::parse(
                raw,
                sh_modinfo.sh_offset as usize,
                sh_modinfo.sh_size as usize,
                b'\0',
            ).context(".modinfo section cannot be parsed as string table")?;

            Ok(Self { strtab: modinfo })
        }

        pub fn get_value(&self, key: &str) -> anyhow::Result<&'a str> {
            for kv in self.strtab.to_vec()? {
                let kv: Vec<&str> = kv.split('=').collect();
                if kv[0] == key {
                    return Ok(kv[1]);
                }
            }
            bail!("Unable to find key {} in modinfo", key)
        }
    }

    fn find_sh_by_name<'a>(
        elf: &'a elf::Elf,
        name: &str,
    ) -> anyhow::Result<&'a goblin::elf::SectionHeader> {
        let shstrtab = &elf.shdr_strtab;
        for sh in &elf.section_headers {
            let sh_name =
                shstrtab.get_at(sh.sh_name).context("Corrupted ELF file")?;
            if sh_name == name {
                return Ok(sh);
            }
        }
        bail!("Unable to find section {}", name);
    }
}

