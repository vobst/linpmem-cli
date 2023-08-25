extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_dir = env::var("OUT_DIR").unwrap();
    let output_file = format!("{}/../../../lib{}", output_dir, package_name);

    cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_language(cbindgen::Language::C)
        .with_include_guard("__PMEM_RUST_H__")
        .generate()
        .expect("Unable to generate C bindings")
        .write_to_file(format!("{}.h", output_file));

    cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_language(cbindgen::Language::Cxx)
        .with_namespace("ffi")
        .with_pragma_once(true)
        .generate()
        .expect("Unable to generate CPP bindings")
        .write_to_file(format!("{}.hpp", output_file));
}
