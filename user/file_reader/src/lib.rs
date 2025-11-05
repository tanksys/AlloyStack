#![no_std]

use alloc::{borrow::ToOwned, string::String};
use as_std::{
    args,
    fs::File,
    io::Read,
    prelude::*,
    println,
    time::{SystemTime, UNIX_EPOCH},
};
use as_std_proc_macro::FaasData;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
// use as_std_proc_macro::FaasData;

#[cfg_attr(feature = "file-based", derive(Serialize, Deserialize))]
#[derive(Default, FaasData)]
struct VecArg {
    #[cfg(feature = "pkey_per_func")]
    content: heapless::String<{ 110 * 1024 * 1024 }>,
    #[cfg(not(feature = "pkey_per_func"))]
    content: String,
}

#[no_mangle]
pub fn main() -> Result<()> {
    let slot_name = args::get("slot_name").unwrap();
    let input_file = args::get("input_file").unwrap();

    println!("file_reader: slot_name: {}", slot_name);
    let mut buffer: DataBuffer<VecArg> = DataBuffer::with_slot(slot_name.to_owned());
    // let mut f = File::open(input_file)?;
    // let mut buf = String::new();
    println!(
        "read_start: {}",
        SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64
    );
    let mut f = File::open(input_file)?;
    #[cfg(not(feature = "pkey_per_func"))]
    {
        let size = f
            .read_to_string(&mut buffer.content)
            .expect("read file failed.");
        println!("file_reader: read_size={}", size);
    }
    #[cfg(feature = "pkey_per_func")]
    {
        let mut data = String::new();
        let size = f.read_to_string(&mut data).expect("read file failed.");
        buffer.content.push_str(&data).unwrap();
        println!("file_reader: read_size={}", size);
    }
    // println!("buffer_long={}", buffer.content.len());
    // f.read_to_string(&mut buf).expect("read file failed.");
    println!(
        "read_end: {}",
        SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64
    );

    Ok(().into())
}
