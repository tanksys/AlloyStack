#![cfg_attr(feature = "with_libos", no_std)]

use as_std::args;

cfg_if::cfg_if! {
    if #[cfg(feature = "with_libos")] {
        use as_std::{agent::FaaSFuncResult as Result, println, libos::libos};
        extern crate alloc;
    } else {
        type Result<T> = core::result::Result<T, String>;
        use std::collections::BTreeMap;
    }
}

#[no_mangle]
pub fn main() -> Result<()> {
    println!("Hello, world! id: {}", args::get("id").unwrap());

    let ret = libos!(addrinfo("localhost"));

    Ok(().into())
}
