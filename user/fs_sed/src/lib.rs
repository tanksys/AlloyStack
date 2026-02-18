#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use as_std::{
    args,
    fs::File,
    io::{Read, Write},
    prelude::*,
    println,
};

/// Read file, replace all occurrences of `pattern` with `replacement`, then
/// write to `output_path` if given, else print to stdout (sed-like behavior).
pub fn sed_file(
    file_path: &str,
    pattern: &str,
    replacement: &str,
    output_path: Option<&str>,
) -> Result<()> {
    let mut f = File::open(file_path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).map_err(|_| "read failed")?;
    let s = core::str::from_utf8(&buf).unwrap_or("");
    let result = s.replace(pattern, replacement);

    if let Some(out) = output_path {
        let mut out_file = File::create(out)?;
        out_file
            .write_all(result.as_bytes())
            .map_err(|_| "write failed")?;
    } else {
        println!("{}", result);
    }
    Ok(().into())
}

#[no_mangle]
pub fn main() -> Result<()> {
    let file_path = args::get("file_path").unwrap_or_else(|| "lines.txt");
    let pattern = args::get("pattern").unwrap_or_else(|| "foo");
    let replacement = args::get("replacement").unwrap_or_else(|| "bar");
    let output_path = args::get("output_path");

    println!(
        "fs_sed: file_path: {}, pattern: {}, replacement: {}",
        file_path, pattern, replacement
    );
    sed_file(file_path, pattern, replacement, output_path.as_deref())?;
    Ok(().into())
}
