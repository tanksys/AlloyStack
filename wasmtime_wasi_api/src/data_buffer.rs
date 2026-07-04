extern crate alloc;

use alloc::{string::String, vec, vec::Vec};
use core::{alloc::Layout, ptr};

use as_hostcall::Verify;
use as_std::libos::libos;
#[cfg(feature = "log")]
use as_std::{
    println,
    time::{SystemTime, UNIX_EPOCH},
};
use as_std_proc_macro::FaasData;
use wasmtime::Caller;

use crate::LibosCtx;

#[derive(FaasData)]
struct WasmDataBuffer(*mut u8, usize);

impl Default for WasmDataBuffer {
    fn default() -> Self {
        Self(core::ptr::null_mut(), Default::default())
    }
}

pub fn buffer_register(
    mut caller: Caller<'_, LibosCtx>,
    slot_name_base: i32,
    slot_name_size: i32,
    buffer_offset: i32,
    buffer_size: i32,
) {
    #[cfg(feature = "log")]
    {
        println!("[Debug] buffer_register");
        println!(
            "[Time] buffer_register: {}",
            SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64
        );
    }

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let mut slot_name: Vec<u8> = Vec::with_capacity(slot_name_size as usize);
    slot_name.resize(slot_name_size as usize, 0);
    memory
        .read(&caller, slot_name_base as usize, &mut slot_name)
        .unwrap();
    let slot_name = String::from_utf8(slot_name).expect("[Err] Not a valid UTF-8 sequence");

    #[cfg(feature = "log")]
    println!("slot_name={}", slot_name);

    let data = memory.data_mut(&mut caller);
    let content = data
        .get_mut(buffer_offset as usize..)
        .and_then(|s| s.get_mut(..buffer_size as usize))
        .unwrap();
    let buffer_base = content.as_mut_ptr();

    #[cfg(feature = "log")]
    {
        let base = data.as_mut_ptr();
        println!(
            "base={:?}, addr={:?}, offset={:?}, size={}",
            base, buffer_base, buffer_offset, buffer_size
        );
    }
    // #[cfg(feature = "log")]
    // println!("content={:?}", content);

    let layout = Layout::new::<WasmDataBuffer>();
    let buffer_addr = libos!(buffer_alloc_raw(layout))
        .expect("failed to allocate wasm buffer metadata");
    unsafe {
        ptr::write(
            buffer_addr as *mut WasmDataBuffer,
            WasmDataBuffer(buffer_base, buffer_size as usize),
        );
    }
    if let Err(error) = libos!(buffer_register(
        &slot_name,
        buffer_addr,
        WasmDataBuffer::__fingerprint(),
        buffer_size as usize
    )) {
        libos!(buffer_dealloc(buffer_addr, layout));
        panic!("failed to register wasm buffer: {:?}", error);
    }
}

pub fn buffer_len(
    mut caller: Caller<'_, LibosCtx>,
    slot_name_base: i32,
    slot_name_size: i32,
) -> i64 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let mut slot_name = vec![0; slot_name_size as usize];
    memory
        .read(&caller, slot_name_base as usize, &mut slot_name)
        .unwrap();
    let slot_name = String::from_utf8(slot_name).expect("[Err] Not a valid UTF-8 sequence");
    libos!(buffer_len(&slot_name))
        .and_then(|len| i64::try_from(len).ok())
        .unwrap_or(-1)
}

pub fn access_buffer(
    mut caller: Caller<'_, LibosCtx>,
    slot_name_base: i32,
    slot_name_size: i32,
    buffer_offset: i32,
    buffer_size: i32,
) {
    #[cfg(feature = "log")]
    {
        println!("[Debug] access_buffer");
        println!(
            "[Time] access_buffer: {}",
            SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64
        );
    }

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let mut slot_name: Vec<u8> = Vec::with_capacity(slot_name_size as usize);
    slot_name.resize(slot_name_size as usize, 0);
    memory
        .read(&caller, slot_name_base as usize, &mut slot_name)
        .unwrap();
    let slot_name = String::from_utf8(slot_name).expect("[Err] Not a valid UTF-8 sequence");

    #[cfg(feature = "log")]
    println!("slot_name={}", slot_name);
    let Some((buffer_addr, fingerprint, data_len)) = libos!(access_buffer(&slot_name))
    else {
        #[cfg(feature = "log")]
        println!("[Debug] access_buffer didn't find slot_name={}", slot_name);

        let data = memory.data_mut(&mut caller);
        data.get_mut(buffer_offset as usize..)
            .and_then(|buffer| buffer.get_mut(..buffer_size as usize))
            .expect("access_buffer target is outside wasm memory")
            .fill(0);
        return;
    };
    if fingerprint != WasmDataBuffer::__fingerprint() {
        libos!(buffer_register(
            &slot_name,
            buffer_addr,
            fingerprint,
            data_len
        ))
        .expect("failed to restore mismatched wasm buffer");
        panic!("wrong wasm buffer fingerprint");
    }
    let wasm_buffer = unsafe { &*(buffer_addr as *const WasmDataBuffer) };

    #[cfg(feature = "log")]
    println!(
        "wasm_buffer -> addr={:?}, size={}",
        wasm_buffer.0, wasm_buffer.1
    );

    if (buffer_size as usize) < data_len {
        panic!(
            "buffer_size={}, data_len={}, access_buffer target is too small",
            buffer_size, data_len
        )
    }

    // access_buffer is a non-consuming read. access_buffer() in the mm service
    // removes the slot while handing out its allocation, so publish the same
    // allocation again before returning. This also lets flag polling observe a
    // value repeatedly until every consumer has seen it.
    libos!(buffer_register(
        &slot_name,
        buffer_addr,
        WasmDataBuffer::__fingerprint(),
        data_len
    ))
    .expect("failed to restore buffer slot after access");

    let buffer = unsafe { core::slice::from_raw_parts(wasm_buffer.0, data_len) };
    // #[cfg(feature = "log")]
    // println!("buffer: {:?}", buffer);
    memory
        .write(&mut caller, buffer_offset as usize, buffer)
        .unwrap();
}
