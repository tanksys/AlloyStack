#![no_std]

#[allow(unused_imports)]
use core::str::FromStr;

use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
};
use hashbrown::HashMap;
pub use as_hostcall::Verify;
use as_std::{
    agent::{DataBuffer, FaaSFuncResult as Result},
    args, println,
    time::{SystemTime, UNIX_EPOCH},
};
use as_std_proc_macro::FaasData;

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

extern crate alloc;

#[cfg_attr(feature = "file-based", derive(Serialize, Deserialize))]
#[derive(Default, FaasData)]
struct Mapper2Reducer {
    #[cfg(feature = "pkey_per_func")]
    shuffle: heapless::FnvIndexMap<heapless::String<32>, u32, 1024>,
    #[cfg(not(feature = "pkey_per_func"))]
    shuffle: HashMap<String, u32>,
}

#[allow(clippy::result_unit_err)]
#[no_mangle]
pub fn main() -> Result<()> {
    let my_id = args::get("id").unwrap();
    let reducer_id: usize = my_id.parse().expect("wrong id.");

    let mapper_num: u64 = args::get("mapper_num")
        .expect("missing arg mapper_num")
        .parse()
        .unwrap_or_else(|_| panic!("bad arg, mapper_num={}", args::get("mapper_num").unwrap()));

    let mut counter: HashMap<String, u32> = HashMap::new();
    println!(
        "access_start: {}",
        SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64
    );
    for i in 0..mapper_num {
        // println!("need databuffer slot={}-{}", i, reducer_id);
        let mapper_result: DataBuffer<Mapper2Reducer> =
            DataBuffer::from_buffer_slot(format!("{}-{}", i, reducer_id))
                .unwrap_or_else(|| panic!("missing mapper result? mapper_id={}", i.to_string()));

        for (word, count) in &mapper_result.shuffle {
            #[cfg(feature = "pkey_per_func")]
            let word = String::from_str(word).unwrap();
            let old_count = *counter.entry(word.to_owned()).or_insert(0);
            #[cfg(feature = "pkey_per_func")]
            counter.insert(word, old_count + count);
            #[cfg(not(feature = "pkey_per_func"))]
            counter.insert(word.to_owned(), old_count + count);
        }
    }
    println!(
        "access_end: {}",
        SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64
    );

    // for (word, count) in counter {
    //     println!("{}:{}", word, count);
    // }
    println!("reducer{} has counted {} words", my_id, counter.len());

    Ok(().into())
}
