#![no_std]
#![allow(internal_features)]
#![feature(lang_items)]
#![feature(linkage)]
#![feature(alloc_error_handler)]
#![feature(ip_in_core)]
#![feature(decl_macro)]
#![feature(concat_idents)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
#![feature(const_mut_refs)]

use agent::FaaSFuncResult;
use alloc::string::String;
#[cfg(feature = "mpk")]
use core::arch::asm;

pub mod agent;
pub mod args;
pub mod console;
pub mod fs;
pub mod init_context;
pub mod io;
pub mod libos;
pub mod mm;
pub mod net;
pub mod prelude;
pub mod sym_patch;
pub mod sync;
pub mod time;

#[cfg(feature = "mpk")]
pub mod mpk;

extern crate alloc;

#[cfg(feature = "alloc_def")]
pub mod heap_alloc;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "unwinding", feature = "panic_def"))] {
        compile_error!("must only choose one in 'unwinding' and 'panic_def'");
    } else
    if #[cfg(feature = "panic_def")] {
        mod panic_def {
            use core::panic::PanicInfo;

            use crate::init_context::isolation_ctx;

            #[panic_handler]
            fn panic_handler(_info: &PanicInfo) -> ! {
                let panic_addr = isolation_ctx().panic_handler;
                crate::println!("panic_handler: {:?}", _info);
                let host_panic_handler: unsafe extern "C" fn() -> ! =
                    unsafe { core::mem::transmute(panic_addr) };
                unsafe { host_panic_handler() }
            }

            #[lang = "eh_personality"]
            extern "C" fn eh_personality() {}
        }
    }
}

#[linkage = "weak"]
#[no_mangle]
pub fn main() -> FaaSFuncResult<()> {
    panic!("need real main");
}

#[no_mangle]
pub extern "C" fn rust_main() -> u64 {
    let mut return_value: Result<(), String> = Ok(());
    #[cfg(feature = "unwinding")]
    {
        let result = unwinding::panic::catch_unwind(|| main());

        match result {
            Ok(func_res) => {
                if let Err(e) = func_res {
                    return_value = Err(alloc::format!("function exec error: {}", e.msg()));
                    return &return_value as *const _ as u64;
                }
            }
            Err(e) => {
                core::mem::forget(e);
                return_value = Err(alloc::format!("catch user function panic."));
                return &return_value as *const _ as u64;
            }
        }
    }
    #[cfg(not(feature = "unwinding"))]
    {
        let result = main();

        if let Err(e) = result {
            // Err(alloc::format!("function exec error: {}", e.msg()))?
            return_value = Err(alloc::format!("function exec error: {}", e.msg()));
            return &return_value as *const _ as u64;
        }
    }
    #[cfg(feature = "mpk")]
    unsafe {
        asm!(
            "wrpkru",
            // return to msvisor. grant privilege. 
            in("rax") 0x0,
            in("rcx") 0,
            in("rdx") 0,
        );
    }
    // return result to pointer
    &return_value as *const _ as u64
}
