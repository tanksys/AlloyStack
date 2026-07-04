#![no_std]
use alloc::{format, vec::Vec};

use as_std::{
    args,
    prelude::*,
    println,
    time::{SystemTime, UNIX_EPOCH},
};
use as_std_proc_macro::FaasData;

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "file-based", derive(Serialize, Deserialize))]
#[derive(Default, FaasData)]
struct VecArg {
    #[cfg(feature = "pkey_per_func")]
    array: heapless::Vec<u32, { 20 * 1024 * 1024 }>,
    #[cfg(not(feature = "pkey_per_func"))]
    array: Vec<u32>,
}

#[cfg(feature = "pkey_per_func")]
#[derive(Default, FaasData)]
struct Pivots {
    array: heapless::Vec<u32, 10>,
}

#[no_mangle]
pub fn main() -> Result<()> {
    let my_id = args::get("id").unwrap();
    let start_micros = timestamp_micros();
    println!(
        "splitter id: {}, com_start2: {}.{:06}",
        my_id,
        start_micros / 1_000_000,
        start_micros % 1_000_000
    );

    let numbers: DataBuffer<VecArg> =
        DataBuffer::from_buffer_slot(format!("sorter-resp-part-{}", my_id)).unwrap();

    #[cfg(feature = "pkey_per_func")]
    let pivots: DataBuffer<Pivots>;
    #[cfg(not(feature = "pkey_per_func"))]
    let pivots: DataBuffer<VecArg>;

    pivots = DataBuffer::from_buffer_slot(format!("pivots-{}", my_id)).unwrap();

    let partitions = split_numbers(&numbers.array, &pivots.array);
    for (idx, partition) in partitions.iter().enumerate() {
        let output_slot = format!("splitter-{}-resp-part-{}", my_id, idx);
        let mut part: DataBuffer<VecArg> = DataBuffer::with_slot(output_slot.clone());
        #[cfg(feature = "pkey_per_func")]
        {
            part.array.resize(partition.len(), 0).unwrap();
            for (idx, item) in part.array.iter_mut().enumerate() {
                *item = partition[idx];
            }
        }
        #[cfg(not(feature = "pkey_per_func"))]
        {
            part.array = partition.clone();
        }
        #[cfg(not(feature = "file-based"))]
        as_std::agent::buffer_set_len(
            &output_slot,
            part.array.len() * core::mem::size_of::<u32>(),
        )
        .expect("failed to set splitter buffer length");
    }

    println!(
        "len of numbers is {}, has split into {} parts: {:?}",
        numbers.array.len(),
        partitions.len(),
        partitions
            .iter()
            .map(|part| part.len())
            .collect::<Vec<usize>>()
    );
    print_timestamp("com_end2");

    Ok(().into())
}

fn timestamp_micros() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).as_micros()
}

fn print_timestamp(label: &str) {
    let micros = timestamp_micros();
    println!("{}: {}.{:06}", label, micros / 1_000_000, micros % 1_000_000);
}

fn split_numbers(numbers: &[u32], pivots: &[u32]) -> Vec<Vec<u32>> {
    let mut result = Vec::new();
    let mut current_start = 0;

    for &pivot in pivots {
        let mut current_partition = Vec::new();

        for &num in &numbers[current_start..] {
            if num < pivot {
                current_partition.push(num);
            } else {
                break;
            }
        }

        result.push(current_partition.clone());
        current_start += current_partition.len();

        if current_start >= numbers.len() {
            break;
        }
    }

    // Add the remaining numbers as the last partition
    result.push(numbers[current_start..].to_vec());

    result
}
