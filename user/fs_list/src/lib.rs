#![no_std]

use alloc::{borrow::ToOwned, string::String};
use as_std::{
    args,
    libos::libos,
    prelude::*,
    println,
    time::{SystemTime, UNIX_EPOCH},
};
use as_std_proc_macro::FaasData;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
// use as_std_proc_macro::FaasData;

pub fn list_directory(dir_name: &str) -> Result<()> {
    let entries = libos!(readdir(dir_name))?;
    println!("entries: {:?}", entries);
    // Use core::mem::forget to avoid dropping Vec<DirEntry> that was allocated
    // on the service-side heap. Dropping it would try to deallocate using the
    // user-side allocator, causing a hang.
    core::mem::forget(entries);
    Ok(().into())
}

#[no_mangle]
pub fn main() -> Result<()> {
    let dir_name = args::get("dir_name").unwrap_or_else(|| ".");
    println!("fs_list: dir_name: {}", dir_name);
    println!(
        "read_start: {}",
        SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64
    );
    // list directory entries using the fdtab readdir hostcall
    list_directory(dir_name).unwrap();
    println!(
        "read_end: {}",
        SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64
    );

    Ok(().into())
}
