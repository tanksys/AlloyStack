#![no_std]

extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[allow(unused_imports)]
use as_std::{
    agent::FaaSFuncResult as Result,
    fs::File,
    io::{Read, Write},
    println,
    time::SystemTime,
};

// fn init_input_file() {
//     File::create("fake_data_0.txt")
//         .unwrap()
//         .write_str(include_str!("../../../image_content/fake_data_0.txt"))
//         .unwrap();

//     File::create("fake_data_1.txt")
//         .unwrap()
//         .write_str(include_str!("../../../image_content/fake_data_1.txt"))
//         .unwrap();

//     File::create("fake_data_2.txt")
//         .unwrap()
//         .write_str(include_str!("../../../image_content/fake_data_2.txt"))
//         .unwrap();

//     File::create("fake_data_3.txt")
//         .unwrap()
//         .write_str(include_str!("../../../image_content/fake_data_3.txt"))
//         .unwrap();

//     File::create("fake_data_4.txt")
//         .unwrap()
//         .write_str(include_str!("../../../image_content/fake_data_4.txt"))
//         .unwrap();

//     let content = include_str!("../../../image_content/sort_data_0.txt");
//     File::create("sort_data_0.txt")
//         .unwrap()
//         .write_str(content)
//         .unwrap();

//     let start = SystemTime::now();
//     let mut array: Vec<i32> = Vec::new();
//     for num in content.split(',') {
//         let num = num.trim();
//         if num.is_empty() {
//             continue;
//         }
//         let num = num.parse().unwrap();
//         array.push(num);
//     }
//     println!(
//         "split {} numbers cost {}ms",
//         array.len(),
//         SystemTime::elapsed(&start).as_millis()
//     );

//     File::create("sort_data_1.txt")
//         .unwrap()
//         .write_str(include_str!("../../../image_content/sort_data_1.txt"))
//         .unwrap();

//     File::create("sort_data_2.txt")
//         .unwrap()
//         .write_str(include_str!("../../../image_content/sort_data_2.txt"))
//         .unwrap();

//     File::create("sort_data_3.txt")
//         .unwrap()
//         .write_str(include_str!("../../../image_content/sort_data_3.txt"))
//         .unwrap();

//     File::create("sort_data_4.txt")
//         .unwrap()
//         .write_str(include_str!("../../../image_content/sort_data_4.txt"))
//         .unwrap();
// }

#[no_mangle]
pub fn main() -> Result<()> {
    // let start_time = SystemTime::now();
    let path = "lines.txt";

    /////////////////// test create/write/read. ///////////////////
    let data = "Rust LibOS Cool.";
    let mut output = File::create(path)?;
    output.write_str(data).expect("");
    // drop(output);

    let mut input_file = File::open(path)?;
    let mut file_content_buf = Vec::new();
    input_file
        .read_to_end(&mut file_content_buf)
        .expect("read failed");

    let file_content = String::from_utf8_lossy(&file_content_buf).to_string();
    println!("file_content: {}", file_content);
    // println!("expect: {}", data);

    assert_eq!(file_content, data);

    /////////////////// test seek. ///////////////////
    input_file.seek(0)?;
    file_content_buf.clear();
    input_file
        .read_to_end(&mut file_content_buf)
        .expect("read failed");

    assert_eq!(
        file_content,
        String::from_utf8_lossy(&file_content_buf).to_string()
    );

    /////////////////// test seek. ///////////////////
    if input_file.metadata().unwrap().st_size != file_content.len() {
        Err("seek failed")?
    }

    // init_input_file();

    // println!(
    //     "simple_file exec: {}ms",
    //     SystemTime::elapsed(&start_time).as_millis()
    // );
    Ok(().into())
}
