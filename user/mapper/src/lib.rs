#![no_std]

#[allow(unused_imports)]
use core::str::FromStr;

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use hashbrown::HashMap;

#[cfg(feature = "pkey_per_func")]
use heapless::FnvIndexMap;

pub use as_hostcall::Verify;
use as_std::{
    agent::{DataBuffer, FaaSFuncResult as Result},
    args,
    fs::File,
    io::Read,
    println,
    time::SystemTime,
};
use as_std_proc_macro::FaasData;

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

extern crate alloc;


#[derive(Default, FaasData)]
#[cfg_attr(feature = "file-based", derive(Serialize, Deserialize))]
struct Reader2Mapper {
    content: String,
}

#[cfg_attr(feature = "file-based", derive(Serialize, Deserialize))]
#[derive(FaasData)]
struct Mapper2Reducer {
    #[cfg(feature = "pkey_per_func")]
    shuffle: heapless::FnvIndexMap<heapless::String<32>, u32, 1024>,
    #[cfg(not(feature = "pkey_per_func"))]
    shuffle: HashMap<String, u32>,
}

impl Default for Mapper2Reducer {
    fn default() -> Self {
        Self {
            #[cfg(feature = "pkey_per_func")]
            shuffle: FnvIndexMap::new(),
            #[cfg(not(feature = "pkey_per_func"))]
            shuffle: HashMap::new(),
        }
    }
}

pub fn getidx(word: &str, reducer_num: u64) -> u64 {
    let mut hash_val: u64 = 0;
    for c in word.chars() {
        hash_val = hash_val * 31 + c as u64;
        hash_val %= reducer_num;
    }
    hash_val
}

fn mapper_func(my_id: &str, reducer_num: u64) -> Result<()> {
    let start = SystemTime::now();

    // let reader: DataBuffer<Reader2Mapper> =
    //     DataBuffer::from_buffer_slot(format!("part-{}", my_id)).expect("missing input data.");
    // println!("access_buffer_long={}", reader.content.len());
    // let content = &reader.content;

    /////// file reader //////
    let file_name = format!("fake_data_{}.txt", my_id);
    let mut f = File::open(&file_name)?;
    let mut content = String::new();
    // let content = get_input_from_mem(&file_name);
    f.read_to_string(&mut content).expect("read file failed.");
    //////////////////////////

    println!(
        "read_end, cost: {}ms",
        SystemTime::elapsed(&start).as_millis()
    );
    let start = SystemTime::now();

    let mut counter = HashMap::new();
    for line in content.lines() {
        let words = line
            .trim()
            .split(' ')
            .filter(|word| word.chars().all(char::is_alphanumeric));

        for word in words {
            let old_count = *counter.entry(word).or_insert(0u32);
            //     for word in word.split('.') {
            //         let old_count = *counter.entry(word).or_insert(0u32);
            //         counter.insert(word, old_count + 1);
            // }
            counter.insert(word, old_count + 1);
        }
    }
    println!(
        "split word end, cost: {}ms",
        SystemTime::elapsed(&start).as_millis()
    );
    let start = SystemTime::now();

    let total: u32 = counter.values().sum();
    // 打印总和
    println!("The sum of all values is: {}", total);
    println!("the counter nums is {}", counter.len());
    let mut data_buffers: Vec<DataBuffer<Mapper2Reducer>> =
        Vec::with_capacity(reducer_num as usize);

    for reducer in 0..reducer_num {
        let mut buffer: DataBuffer<Mapper2Reducer> =
            DataBuffer::with_slot(format!("{}-{}", my_id, reducer));
        buffer.shuffle = Default::default();
        data_buffers.push(buffer);
    }

    for (word, count) in counter {
        let shuffle_idx = getidx(word, reducer_num);

        #[cfg(feature = "pkey_per_func")]
        let word = heapless::String::from_str(word).unwrap();
        #[cfg(not(feature = "pkey_per_func"))]
        let word = word.to_string();

        if let Some(buffer) = data_buffers.get_mut(shuffle_idx as usize) {
            let _ = buffer.shuffle.insert(word, count);
        } else {
            panic!("vec get_mut failed, idx={}", shuffle_idx)
        }
    }

    println!(
        "shuffle end, cost {}ms",
        SystemTime::elapsed(&start).as_millis()
    );

    Ok(().into())
}

#[allow(clippy::result_unit_err)]
#[no_mangle]
pub fn main() -> Result<()> {
    let my_id = args::get("id").unwrap();
    let reducer_num: u64 = args::get("reducer_num")
        .expect("missing arg reducer_num")
        .parse()
        .unwrap_or_else(|_| panic!("bad arg, reducer_num={}", args::get("reducer_num").unwrap()));

    mapper_func(my_id, reducer_num)
}

// fn get_input_from_mem(filename: &str) -> &'static str {
//     match filename {
//         "fake_data_0.txt" => {
//             include_str!("../../../image_content/fake_data_0.txt")
//         }
//         "fake_data_1.txt" => {
//             include_str!("../../../image_content/fake_data_1.txt")
//         }
//         "fake_data_2.txt" => {
//             include_str!("../../../image_content/fake_data_2.txt")
//         }
//         "fake_data_3.txt" => {
//             include_str!("../../../image_content/fake_data_3.txt")
//         }
//         "fake_data_4.txt" => {
//             include_str!("../../../image_content/fake_data_4.txt")
//         }
//         _ => {
//             panic!("unknown filename")
//         }
//     }
// }
