#![no_std]

use alloc::{string::ToString, vec::Vec};
use as_std::{agent::FaaSFuncResult as Result, args, println};
use wasmtime::{component::Func, Engine, Extern, Instance, Linker, Module, Store};
use wasmtime::{AsContext, Caller};
mod capis;
extern crate alloc;

static CWASM: &[u8] = include_bytes!("../add.cwasm");

#[no_mangle]
pub fn main() -> Result<()> {
    let id = args::get("id").unwrap();
    println!("wasmtiem hello, world! id: {}", id);

    let engine = Engine::default();
    let module = unsafe { Module::deserialize(&engine, CWASM) }.map_err(|e| e.to_string())?;

    let mut linker = Linker::new(&engine);
    linker
        .func_wrap(
            "wasi1",
            "fdclose",
            |mut caller: Caller<'_, ()>, a: i32, b: i64| -> (i64, i32) {
                let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                let mut buf = [0; 64];
                memory.read(caller, 100, &mut buf);
                println!("{:?}", buf);
                (b + 1, a + 1)
            },
        )
        .unwrap();

    let mut store = Store::new(&engine, ());

    let instance = linker.instantiate(&mut store, &module)?;

    let func = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, "add")
        .map_err(|e| e.to_string())?;

    let ret = func.call(store, (1, 2)).map_err(|e| e.to_string())?;
    println!("wasmtime ret: {}", ret);

    Ok(().into())
}
