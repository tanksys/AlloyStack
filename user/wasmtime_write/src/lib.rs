#![no_std]

extern crate alloc;
use alloc::string::ToString;
use spin::Mutex;

use as_std::{agent::FaaSFuncResult as Result, args, println, libos::libos};

use wasmtime_wasi_api::{wasmtime, LibosCtx};
use wasmtime::Store;

static CWASM: &[u8] = include_bytes!("../write.cwasm");

static INIT_LOCK: Mutex<()> = Mutex::new(());

#[no_mangle]
pub fn main() -> Result<()> {
    let id = args::get("id").unwrap();
    println!("rust: write_{} begin", id);
    
    let lock = INIT_LOCK.lock();
    let (engine, module, linker) = wasmtime_wasi_api::build_wasm(CWASM);
    drop(lock);

    let mut store = Store::new(&engine, LibosCtx{id: id.to_string()});
    let instance = linker.instantiate(&mut store, &module)?;

    let main = instance
        .get_typed_func::<(), ()>(&mut store, "_start")
        .map_err(|e| e.to_string())?;

    main.call(store, ()).map_err(|e| e.to_string())?;

    println!("rust: write_{} end", id);
    Ok(().into())
}
