#![no_std]
use alloc::{format, vec::Vec};

use as_std::{
    args,
    prelude::*,
    time::{SystemTime, UNIX_EPOCH},
};
use as_std_proc_macro::FaasData;

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[cfg(feature = "pkey_per_func")]
type NumberArray = heapless::Vec<u32, { 20 * 1024 * 1024 }>;
#[cfg(not(feature = "pkey_per_func"))]
type NumberArray = Vec<u32>;

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
    print_timestamp("com_start3");
    let my_id = args::get("id").unwrap();
    let sorter_num: u32 = {
        let m = args::get("sorter_num").unwrap();
        m.parse().unwrap()
    };

    let partitions: Vec<DataBuffer<VecArg>> = (0..sorter_num)
        .map(|idx| {
            DataBuffer::from_buffer_slot(format!("splitter-{}-resp-part-{}", idx, my_id)).unwrap()
        })
        .collect();

    let output_slot = format!("merge_result_{}", my_id);
    let mut merged_result: DataBuffer<VecArg> = DataBuffer::with_slot(output_slot.clone());

    merge_partitions(
        partitions.iter().map(|buffer| &buffer.array).collect(),
        &mut merged_result.array,
    );
    #[cfg(not(feature = "file-based"))]
    as_std::agent::buffer_set_len(
        &output_slot,
        merged_result.array.len() * core::mem::size_of::<u32>(),
    )
    .expect("failed to set merger buffer length");
    // println!("merged_result: {:?}", merged_result);
    print_timestamp("com_end3");
    Ok(().into())
}

fn print_timestamp(label: &str) {
    let micros = SystemTime::now().duration_since(UNIX_EPOCH).as_micros();
    println!("{}: {}.{:06}", label, micros / 1_000_000, micros % 1_000_000);
}

fn merge_partitions(partitions: Vec<&NumberArray>, dst: &mut NumberArray) {
    let mut indices: Vec<usize> = partitions.iter().map(|_| 0).collect();

    loop {
        let mut min_value = core::u32::MAX;
        let mut min_partition = None;

        for (i, &index) in indices.iter().enumerate() {
            if index < partitions[i].len()
                && *partitions[i].get(index).unwrap_or(&u32::MAX) < min_value
            {
                min_value = partitions[i][index];
                min_partition = Some(i);
            }
        }

        match min_partition {
            Some(partition_idx) => {
                #[allow(unused_variables)]
                let ret = dst.push(min_value);
                #[cfg(feature = "pkey_per_func")]
                {
                    ret.unwrap();
                }
                indices[partition_idx] += 1;
            }
            None => break,
        }
    }
}
