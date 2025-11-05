#![cfg_attr(feature = "with_libos", no_std)]

use core::time::Duration;

use as_std::time::sleep;

cfg_if::cfg_if! {
    if #[cfg(feature = "with_libos")] {
        use as_std::{agent::FaaSFuncResult as Result};
        extern crate alloc;
    } else {
        type Result<T> = core::result::Result<T, String>;
    }
}

#[no_mangle]
pub fn main() -> Result<()> {
    sleep(Duration::from_secs(100000));

    Ok(().into())
}
