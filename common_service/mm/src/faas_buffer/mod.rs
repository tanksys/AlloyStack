extern crate alloc;

use core::{alloc::Layout, ptr::NonNull};

use alloc::{borrow::ToOwned, string::String};

use as_hostcall::{
    mm::{MMError, MMResult},
    SERVICE_HEAP_SIZE,
};
use linked_list_allocator::LockedHeap;

use hashbrown::{hash_map::Entry, HashMap};
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref BUFFER_REGISTER: Mutex<HashMap<String, (usize, u64)>> = Mutex::new(HashMap::new());
    static ref BUFFER_ALLOCATOR: LockedHeap = unsafe {
        LockedHeap::new(
            as_std::init_context::ISOLATION_CTX.lock().heap_range.1 as *mut u8,
            SERVICE_HEAP_SIZE / 2,
        )
    };
}

#[no_mangle]
pub fn buffer_alloc_raw(l: Layout) -> MMResult<usize> {
    let addr = BUFFER_ALLOCATOR
        .lock()
        .allocate_first_fit(l)
        .map_err(|_| MMError::NoMemory)?;

    Ok(addr.as_ptr() as usize)
}

#[no_mangle]
pub fn buffer_register(slot: &str, addr: usize, fingerprint: u64) -> MMResult<()> {
    let mut register = BUFFER_REGISTER.lock();
    match register.entry(slot.to_owned()) {
        Entry::Vacant(entry) => {
            entry.insert((addr, fingerprint));
            Ok(())
        }
        Entry::Occupied(_) => Err(MMError::BufferSlotExists),
    }
}

#[no_mangle]
pub fn buffer_alloc(slot: &str, l: Layout, fingerprint: u64) -> MMResult<usize> {
    let addr = buffer_alloc_raw(l)?;
    if let Err(error) = buffer_register(slot, addr, fingerprint) {
        buffer_dealloc(addr, l);
        return Err(error);
    }
    Ok(addr)
}

#[no_mangle]
pub fn access_buffer(slot: &str) -> Option<(usize, u64)> {
    let mut register = BUFFER_REGISTER.lock();
    // as_std::println!("buffer register: ");
    // for (k, v) in register.iter() {
    //     as_std::println!("  {}: {:?}", k, v);
    // }
    register.remove(slot)
}

#[no_mangle]
pub fn borrow_buffer(slot: &str) -> Option<(usize, u64)> {
    BUFFER_REGISTER.lock().get(slot).copied()
}

#[no_mangle]
pub fn buffer_dealloc(addr: usize, l: Layout) {
    unsafe {
        BUFFER_ALLOCATOR
            .lock()
            .deallocate(NonNull::new(addr as *mut u8).unwrap(), l)
    }
}
