#![no_std]

extern crate alloc;
use core::mem::forget;

use alloc::{string::{String, ToString}, vec::Vec, borrow::ToOwned};
use spin::Mutex;

use as_hostcall::types::{OpenFlags, OpenMode};
use as_std::{agent::{DataBuffer, FaaSFuncResult as Result}, args, println, libos::libos, time::{SystemTime, UNIX_EPOCH}};
use as_std_proc_macro::FaasData;

use wasmtime_wasi_api::{wasmtime, LibosCtx};
use wasmtime::Store;

static CWASM: &[u8] = include_bytes!("../func.cwasm");

static INIT_LOCK: Mutex<()> = Mutex::new(());

#[allow(dead_code)]
#[derive(FaasData)]
pub struct MyData {
    pub current_time: SystemTime,
}
impl Default for MyData {
    fn default() -> Self {
        Self {
            current_time: SystemTime::now(),
        }
    }
}

lazy_static::lazy_static! {
    static ref MUST_OPEN_ROOT: bool = {
        libos!(open("/", OpenFlags::empty(), OpenMode::RD)).unwrap();
        true
    };
}

fn func_body(my_id: &str, func_num: u64) -> Result<()> {
    // #[cfg(feature = "log")]
    println!("rust: my_id: {:?}, func_num: {:?}", my_id, func_num);

    let wasi_args: Vec<String> = Vec::from([
        "fake system path!".to_string(),
        my_id.to_string(),
        func_num.to_string(),
    ]);
    wasmtime_wasi_api::set_wasi_args(my_id, wasi_args);

    let _open_root = *MUST_OPEN_ROOT;

    let lock = INIT_LOCK.lock();
    let (engine, module, linker) = wasmtime_wasi_api::build_wasm(CWASM);
    drop(lock);

    let mut store = Store::new(&engine, LibosCtx{id: my_id.to_string()});
    let instance = linker.instantiate(&mut store, &module)?;

    let main = instance
        .get_typed_func::<(), ()>(&mut store, "_start")
        .map_err(|e| e.to_string())?;

    main.call(store, ()).map_err(|e| e.to_string())?;

    
    if func_num == 9 {
        let data = DataBuffer::<MyData>::from_buffer_slot("Conference".to_owned());
        if let Some(buffer) = data {
            let dur = buffer.current_time.elapsed();
            println!("end to end latency: {:?}", dur);
        } else {
            Err("buffer is none")?
        }
    }
    Ok(().into())
}

#[no_mangle]
pub fn main() -> Result<()> {
    let my_id = args::get("id").unwrap();
    let func_num: u64 = args::get("func_num")
        .expect("missing arg func_num")
        .parse()
        .unwrap_or_else(|_| panic!("bad arg, func_num={}", args::get("func_num").unwrap()));
    if func_num == 0 {
        let mut d = DataBuffer::<MyData>::with_slot("Conference".to_owned());
        d.current_time = SystemTime::now();
        // let start_time = SystemTime::now().duration_since(UNIX_EPOCH).as_millis();
        // println!("start_time: {:?}", start_time);
    }
    
    func_body(my_id, func_num)
}
