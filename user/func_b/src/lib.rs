#![no_std]

extern crate alloc;

use alloc::{borrow::ToOwned, vec::Vec};
use as_std::{
    agent::{DataBuffer, FaaSFuncResult as Result},
    println,
    time::SystemTime,
};
use as_std_proc_macro::FaasData;

const DATA_SIZE: usize = include!("../../data_size.config") / 8;

#[derive(FaasData, serde::Serialize, serde::Deserialize)]
struct MyComplexData {
    data: Vec<u64>,
}

impl Default for MyComplexData {
    fn default() -> Self {
        Self {
            data: Vec::with_capacity(DATA_SIZE),
        }
    }
}

#[no_mangle]
#[allow(clippy::result_unit_err)]
pub fn main() -> Result<()> {
    // println!("func b");
    let func_b_start = SystemTime::now();
    let data = DataBuffer::<MyComplexData>::from_buffer_slot("Conference".to_owned());
    if let Some(buffer) = data {
        let data_size = buffer.data.len();
        for i in 0..buffer.data.len() {
            let _ = unsafe { core::ptr::read_volatile((&buffer.data[i]) as *const u64) };
        }
        core::mem::forget(buffer);
        println!(
            "data size: {} bytes, cost {} ns",
            data_size * 8,
            SystemTime::now().duration_since(func_b_start).as_nanos()
        );
        Ok(().into())
    } else {
        Err("buffer is none")?
    }
}
