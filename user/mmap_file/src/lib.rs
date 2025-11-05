#![no_std]
use alloc::string::String;
use as_std::{agent::FaaSFuncResult as Result, fs::File, mm::Mmap, println};

extern crate alloc;

#[no_mangle]
pub fn main() -> Result<()> {
    let file1 = File::open("lines.txt").expect("file1 don't exist?");
    let file2 = File::open("fake_data_0.txt").expect("file2 don't exist?");

    let mmap_area1 = Mmap::mmap_file(file1).unwrap();
    let mmap_area2 = Mmap::mmap_file(file2).unwrap();
    // println!("successfully libos mmap");

    println!(
        "mmap_area2 content is: {}",
        String::from_utf8_lossy(mmap_area2.as_ref())
    );
    println!(
        "mmap_area1 content is: {}",
        String::from_utf8_lossy(mmap_area1.as_ref())
    );

    Ok(().into())
}
