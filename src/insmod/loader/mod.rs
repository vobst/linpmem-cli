use std::error::Error;
use std::fs;

/// Searches for a module that is likely to be compatible to the running kernel.
pub fn find_valid_module() -> Result<fs::File, &'static str> {
    Err("Module search is not implemented")
}

/// Loads a, potentially compressed, module from a file into a buffer
pub fn mod_to_vec_decompress(
    _module: fs::File,
) -> Result<Vec<u8>, &'static str> {
    Err("Module decompression is not implemented")
}

/// Performs various adjustments to a module in order to make it loadable for
/// the current kernel.
pub fn adjust_module(
    module: &fs::File,
    _valid_module: &Vec<u8>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let adjusted_module = mod_to_vec_decompress(module.try_clone()?)?;

    Ok(adjusted_module)
}
