//! This crate support the defination of symbols `set_handler_addr`
//! and `get_handler_addr`. These two symbols will be lookup when
//! initialize service.
//! To some service that depent `std`, they would use this crate directly
//! rather than `as_std`.

use as_hostcall::{types::HostCallResult as HCResult, IsolationContext};

use spin::Mutex;

pub static ISOLATION_CTX: Mutex<IsolationContext> = Mutex::new(IsolationContext::uninit());

/// This is a non-pub function because it should not be init in other file.
fn isolation_ctx_mut() -> spin::MutexGuard<'static, IsolationContext> {
    ISOLATION_CTX.lock()
}

pub fn isolation_ctx() -> spin::MutexGuard<'static, IsolationContext> {
    let ctx = ISOLATION_CTX.lock();
    if ctx.find_handler == 0 {
        panic!("uninit")
    }
    ctx
}

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn set_handler_addr(ctx: &IsolationContext) -> HCResult {
    let mut isol_ctx = isolation_ctx_mut();
    if isol_ctx.find_handler != 0 && isol_ctx.find_handler != ctx.find_handler {
        panic!();
        // return Err(HCError::HasBeenSet);
    };
    *isol_ctx = ctx.clone();

    #[cfg(feature = "alloc_def")]
    {
        use crate::heap_alloc::init_heap;
        init_heap(ctx.heap_range.0);
    }

    Ok(())
}

#[no_mangle]
pub extern "C" fn get_handler_addr() -> usize {
    let isol_ctx = isolation_ctx();

    isol_ctx.find_handler
}
