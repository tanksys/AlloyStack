#![cfg_attr(feature = "with_libos", no_std)]

use as_std::args;

cfg_if::cfg_if! {
    if #[cfg(feature = "with_libos")] {
        use as_std::{agent::FaaSFuncResult as Result, println};
        extern crate alloc;
    } else {
        type Result<T> = core::result::Result<T, String>;
        use std::collections::BTreeMap;
    }
}

// static TEXT: &str = include_str!("../fake_data_0.txt");

#[no_mangle]
pub fn main() -> Result<()> {
    let id = args::get("id").unwrap();
    println!("Hello, world! id: {}", id);
    #[cfg(feature = "measure_mem")]
    {
        use as_std::libos::MetricEvent::Mem;
        as_std::libos::metric(Mem);
    }

    Ok(().into())
}
