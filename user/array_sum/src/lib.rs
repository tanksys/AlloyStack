#![no_std]

extern crate alloc;

// use alloc::vec::Vec;
use alloc::{format, string::String};
// use as_std::{
//     agent::{DataBuffer, FaaSFuncResult as Result},
//     println,
//     time::{SystemTime, UNIX_EPOCH},
// };
use as_std::{
    agent::{DataBuffer, FaaSFuncResult as Result},
    args, println,
};
use as_std_proc_macro::FaasData;
use serde::{Deserialize, Serialize};

#[derive(Default, FaasData, Serialize, Deserialize)]
struct Arraysum {
    raw_data: String,
    count: i32,
}

#[allow(clippy::identity_op)]
const DATA_SIZE: usize = include!("../../function_chain_data_size.config");

#[allow(clippy::result_unit_err)]
#[no_mangle]
pub fn main() -> Result<()> {
    let n = args::get("n").expect("missing arg 'n'?");
    let i: i32 = n.parse().expect("wrong arg 'n' format");

    let previous_cnt: i32 = if i == 0 {
        0
    } else {
        DataBuffer::<Arraysum>::from_buffer_slot(format!("slot_{}", i - 1))
            .expect("missing data buffer?")
            .count
    };

    let mut next_buffer: DataBuffer<Arraysum> = DataBuffer::with_slot(format!("slot_{}", n));
    next_buffer.raw_data = n.repeat(DATA_SIZE);
    next_buffer.count = previous_cnt + 1;
    println!("count is {}", next_buffer.count);

    Ok(().into())
}
