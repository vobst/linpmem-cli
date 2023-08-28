use anyhow::{bail, Context};
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
            bail!("Module {} has unkown compression", name)
        }
    }
}

/// Searches for a module that is likely to be compatible to the running kernel.
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

/// Performs various adjustments to a module in order to make it loadable for
/// the current kernel.
pub fn adjust_module(
    module: &fs::File,
    _valid_module: &Vec<u8>,
) -> anyhow::Result<Vec<u8>> {
    let mut adjusted_module = Vec::new();
    module
        .try_clone()?
        .read_to_end(&mut adjusted_module)
        .context("Failed to read the module to adjust")?;

    Ok(adjusted_module)
}
