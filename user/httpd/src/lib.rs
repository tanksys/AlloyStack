#![no_std]
#![allow(clippy::result_unit_err)]

extern crate alloc;

use alloc::format;
use as_std::{
    agent::FaaSFuncResult as Result,
    net::{TcpListener, TcpStream}, println,
};

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 4096];
    let _n = stream.read(&mut buffer).expect("tcp connect read failed");

    // println!("Request: {:#?}", String::from_utf8_lossy(&buffer[..n]));

    let content = "hello";
    let resp = [
        "HTTP/1.1 200 OK\r\n",
        &format!("Content-Length: {}\r\n", content.len()),
        "Date: Tue, 05 Sep 2023 06:52:35 GMT\r\n",
        "\r\n",
        content,
    ];
    stream.write_all(resp.join("").as_bytes()).unwrap();
}

#[no_mangle]
pub fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    if let Some(stream) = listener.incoming().next() {
        // println!("Connection established!");
        handle_connection(stream);
    }

    println!("httpd finish.");
    Ok(().into())
}
