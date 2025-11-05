#![no_std]
#![allow(clippy::result_unit_err)]

use as_std::{
    io::{Read, Write},
    net::TcpStream,
    prelude::*,
};

#[no_mangle]
pub fn main() -> Result<()> {
    let mut stream = TcpStream::connect("example.com")?;
    println!("tcp connection created.");
    stream.write_all(b"GET / HTTP/1.0\r\n\r\n")?;
    let mut buffer = [0; 4096];
    // println!("send data finish.");
    loop {
        let n = stream.read(&mut buffer).expect("read failed");
        if n == 0 {
            break;
        }
        let response = String::from_utf8_lossy(&buffer[..n]);
        println!("{}", response);
    }

    Ok(().into())
}
