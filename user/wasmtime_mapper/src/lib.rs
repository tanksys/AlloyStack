#![no_std]

extern crate alloc;
use core::mem::forget;

use alloc::{string::{String, ToString}, vec::Vec};
use spin::Mutex;

use as_hostcall::types::{OpenFlags, OpenMode};
use as_std::{agent::FaaSFuncResult as Result, args, libos::libos};


use wasmtime_wasi_api::{wasmtime, LibosCtx};
use wasmtime::Store;

static CWASM: &[u8] = include_bytes!("../mapper.cwasm");

static INIT_LOCK: Mutex<()> = Mutex::new(());

lazy_static::lazy_static! {
    static ref MUST_OPEN_ROOT: bool = {
        libos!(open("/", OpenFlags::empty(), OpenMode::RD)).unwrap();
        true
    };
}

fn func_body(my_id: &str, reducer_num: u64) -> Result<()> {
    #[cfg(feature = "log")]

    println!("rust: my_id: {:?}, reducer_num: {:?}", my_id, reducer_num);

    let wasi_args: Vec<String> = Vec::from([
        "fake system path!".to_string(),
        my_id.to_string(),
        reducer_num.to_string(),
    ]);
    wasmtime_wasi_api::set_wasi_args(my_id, wasi_args);

    let _open_root = *MUST_OPEN_ROOT;

    let lock = INIT_LOCK.lock();
    let (engine, module, linker) = wasmtime_wasi_api::build_wasm(CWASM);
    drop(lock);

    let mut store = Store::new(&engine, LibosCtx{id: my_id.to_string()});
    let instance = linker.instantiate(&mut store, &module)?;

    let memory = instance.get_memory(&mut store, "memory").unwrap();
    let _pages = memory.grow(&mut store, 20000).unwrap();

    let main = instance
        .get_typed_func::<(), ()>(&mut store, "_start")
        .map_err(|e| e.to_string())?;

    main.call(&mut store, ()).map_err(|e| e.to_string())?;
    forget(store);
    #[cfg(feature = "log")]
    println!("rust: wasmtime_mapper_{:?} finished!", my_id);

    Ok(().into())
}

#[no_mangle]
pub fn main() -> Result<()> {
    let my_id = args::get("id").unwrap();
    let reducer_num: u64 = args::get("reducer_num")
        .expect("missing arg reducer_num")
        .parse()
        .unwrap_or_else(|_| panic!("bad arg, reducer_num={}", args::get("reducer_num").unwrap()));

    func_body(my_id, reducer_num)
}
