#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use as_std::{
    args,
    fs::File,
    io::Read,
    prelude::*,
    println,
};

/// Read a file and print its contents to stdout (cat-like behavior).
pub fn cat_file(file_path: &str) -> Result<()> {
    let mut f = File::open(file_path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).map_err(|_| "read failed")?;
    let s = core::str::from_utf8(&buf).unwrap_or("");
    println!("{}", s);
    Ok(().into())
}

#[no_mangle]
pub fn main() -> Result<()> {
    let file_path = args::get("file_path").unwrap_or_else(|| "lines.txt");
    println!("fs_cat: file_path: {}", file_path);
    cat_file(file_path)?;
    Ok(().into())
}
