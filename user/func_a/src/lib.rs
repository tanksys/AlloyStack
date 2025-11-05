#![no_std]

extern crate alloc;

use alloc::{borrow::ToOwned, vec::Vec};

#[allow(unused_imports)]
use as_std::{
    agent::{DataBuffer, FaaSFuncResult as Result},
    println,
    time::{SystemTime, UNIX_EPOCH},
};
use as_std_proc_macro::FaasData;

// const DATA_SIZE: usize = 1024 * 1024 * 256 / 8;
const DATA_SIZE: usize = include!("../../data_size.config") / 8;

#[derive(FaasData, serde::Serialize, serde::Deserialize)]
struct VecArg {
    data: Vec<u64>,
}

impl Default for VecArg {
    fn default() -> Self {
        let data = Vec::with_capacity(DATA_SIZE);
        Self { data }
    }
}

#[allow(clippy::result_unit_err)]
#[no_mangle]
pub fn main() -> Result<()> {
    let mut d = DataBuffer::<VecArg>::with_slot("Conference".to_owned());
    d.data.resize(DATA_SIZE, 0);

    for (idx, val) in &mut d.data.iter_mut().enumerate() {
        *val = (idx % (u64::MAX - 1) as usize) as u64
    }

    println!("write {} bytes", d.data.len() * 8);
    // let register_start = SystemTime::now().duration_since(UNIX_EPOCH).as_nanos();
    // let result = DataBuffer::<VecArg>::from_buffer_slot("Conference".to_owned());

    // if let Some(buffer) = result {
    //     for i in 0..buffer.data.len() {
    //         let _ = unsafe { core::ptr::read_volatile((&buffer.data[i]) as *const u64) };
    //     }
    // }
    // let access_end2 = SystemTime::now().duration_since(UNIX_EPOCH).as_nanos();

    // println!("phase34_dur={}", access_end2 - register_start);

    Ok(().into())
}
