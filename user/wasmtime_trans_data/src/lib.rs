#![no_std]

extern crate alloc;
use core::mem::forget;

use alloc::{string::{String, ToString}, vec::Vec};
use spin::Mutex;

use as_hostcall::types::{OpenFlags, OpenMode};
use as_std::{agent::FaaSFuncResult as Result, args, println, libos::libos, time::{SystemTime, UNIX_EPOCH}};


use wasmtime_wasi_api::{wasmtime, LibosCtx};
use wasmtime::Store;

static CWASM: &[u8] = include_bytes!("../trans_data.cwasm");

static INIT_LOCK: Mutex<()> = Mutex::new(());

lazy_static::lazy_static! {
    static ref MUST_OPEN_ROOT: bool = {
        libos!(open("/", OpenFlags::empty(), OpenMode::RD)).unwrap();
        true
    };
}

fn func_body() -> Result<()> {
    let _open_root = *MUST_OPEN_ROOT;

    let lock = INIT_LOCK.lock();
    let (engine, module, linker) = wasmtime_wasi_api::build_wasm(CWASM);
    drop(lock);

    let mut store = Store::new(&engine, LibosCtx{id: "0".to_string()});
    let instance = linker.instantiate(&mut store, &module)?;
    
    let memory = instance.get_memory(&mut store, "memory").unwrap();
    let pages = memory.grow(&mut store, 20000).unwrap();
    println!("rust: pages: {}", pages);

    let main = instance
        .get_typed_func::<(), ()>(&mut store, "_start")
        .map_err(|e| e.to_string())?;
    
    main.call(store, ()).map_err(|e| e.to_string())?;
    
    Ok(().into())
}

#[no_mangle]
pub fn main() -> Result<()> {
    func_body()
}
