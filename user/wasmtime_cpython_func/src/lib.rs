#![no_std]

extern crate alloc;

use alloc::sync::Arc;
use alloc::{format, string::{String, ToString}, vec::Vec};
use spin::Mutex;
use core::mem::forget;

use as_hostcall::types::{OpenFlags, OpenMode};
use as_std::{agent::FaaSFuncResult as Result, args, libos::libos, println, time::{SystemTime, UNIX_EPOCH},};

use wasmtime_wasi_api::{wasmtime, LibosCtx};
use wasmtime::Store;

const CWASM: &[u8] = include_bytes!("../python.cwasm");

static INIT_LOCK: Mutex<()> = Mutex::new(());

lazy_static::lazy_static! {
    static ref MUST_OPEN_ROOT: bool = {
        libos!(open("/", OpenFlags::empty(), OpenMode::RD)).unwrap();
        true
    };
}

fn func_body(pyfile_path: &str, func_num: u64) -> Result<()> {
    let my_map_id = &format!("func_{}", func_num);

    let mut jmpbuf = sjlj::JumpBuf::new();
    let if_panic = unsafe { sjlj::setjmp(&mut jmpbuf) };
    if if_panic != 0 {
        #[cfg(feature = "log")]
        println!("[Info] normal exit. if_panic: {:?}", if_panic);
        return Ok(().into());
    } else {
        wasmtime_wasi_api::JMP_BUF_MAP.lock().insert(my_map_id.to_string(), Arc::new(jmpbuf));
    }

    #[cfg(feature = "log")]
    println!("rust: pyfile_path: {:?}, func_num: {:?}", pyfile_path, func_num);

    let wasi_args: Vec<String> = Vec::from([
        "python.wasm".to_string(),
        pyfile_path.to_string(),
        func_num.to_string(),
    ]);
    wasmtime_wasi_api::set_wasi_args(my_map_id, wasi_args);

    let _open_root = *MUST_OPEN_ROOT;

    let lock = INIT_LOCK.lock();
    let (engine, module, linker) = wasmtime_wasi_api::build_wasm(CWASM);
    drop(lock);

    let mut store = Store::new(&engine, LibosCtx{id: my_map_id.to_string()});
    let instance = linker.instantiate(&mut store, &module)?;

    let main = instance
        .get_typed_func::<(), ()>(&mut store, "_start")
        .map_err(|e| e.to_string())?;

    // println!("phase0: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    main.call(store, ()).map_err(|e| e.to_string())?;

    #[cfg(feature = "log")]
    println!("rust: wasmtime_cpython_func_{:?} finished!", my_id);

    Ok(().into())
}


#[no_mangle]
pub fn main() -> Result<()> {
    let pyfile_path = args::get("pyfile_path").unwrap();
    let func_num: u64 = args::get("func_num")
        .expect("missing arg func_num")
        .parse()
        .unwrap_or_else(|_| panic!("bad arg, func_num={}", args::get("func_num").unwrap()));

    func_body(pyfile_path, func_num)
}
