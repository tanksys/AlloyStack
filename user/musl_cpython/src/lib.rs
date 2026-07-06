#![no_std]

// CPython behaves as a process-lifetime runtime. Until its allocator teardown
// is isolated from the Rust app heap, avoid post-main allocation and flushing.
as_musl::entry_no_flush!(alloy_c_main);
