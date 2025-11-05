use linked_list_allocator::LockedHeap;
use as_hostcall::SERVICE_HEAP_SIZE;

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Currently, all service will get a static heap region. It is work well but
/// maybe cause wasting memory.
pub fn init_heap(heap_start: usize) {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(heap_start as *mut u8, SERVICE_HEAP_SIZE)
    }
}

#[alloc_error_handler]
/// panic when heap allocation error occurs
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    let (used_mem, free_mem) = {
        let alloctor = HEAP_ALLOCATOR.lock();
        (alloctor.used(), alloctor.free())
    };

    panic!(
        "Heap allocation error, layout = {:?}, used mem={}KB, free mem={}KB",
        layout,
        used_mem >> 10,
        free_mem >> 10
    );
}
