#![no_std]
#![feature(is_sorted)]

use alloc::{format, vec::Vec};

use as_std::{
    args,
    prelude::*,
    time::{SystemTime, UNIX_EPOCH},
};
use as_std_proc_macro::FaasData;

#[cfg_attr(feature = "file-based", derive(Serialize, Deserialize))]
#[derive(Default, FaasData)]
struct VecArg {
    #[cfg(feature = "pkey_per_func")]
    array: heapless::Vec<u32, { 20 * 1024 * 1024 }>,
    #[cfg(not(feature = "pkey_per_func"))]
    array: Vec<u32>,
}

#[no_mangle]
pub fn main() -> Result<()> {
    println!(
        "com_start4: {}",
        SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64
    );
    let merger_num: u32 = {
        let m = args::get("merger_num").unwrap();
        m.parse().unwrap()
    };

    let merged_results = (0..merger_num).map(|idx| {
        DataBuffer::<VecArg>::from_buffer_slot(format!("merge_result_{}", idx)).unwrap()
    });

    let mut last_max = u32::MIN;
    let mut counter = 0;
    for array in merged_results {
        counter += array.array.len();
        if !array.array.is_sorted() {
            Err("sort result is wrong")?
        }

        if let Some(first) = array.array.first() {
            if *first < last_max {
                Err("sort result is wrong")?
            }
            last_max = *first
        }
    }
    println!(
        "com_end4: {}",
        SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64
    );
    println!("sort result is ok, total sort {} numbers", counter);
    Ok(().into())
}
