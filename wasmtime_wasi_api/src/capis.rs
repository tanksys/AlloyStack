#![allow(non_camel_case_types)]

extern crate sjlj;
use core::ptr::{null, null_mut};

use as_hostcall::{mm::{MMResult, ProtFlags}, signal::{SigAction, SigInfo, Sigset, Ucontext}};
use as_std::libos::libos;
use as_std::println;
use sjlj::{JumpBuf, longjmp, setjmp};

// Flags to either `wasmtime_mmap_{new,remap}` or `wasmtime_mprotect`.

/// Indicates that the memory region should be readable.
pub const WASMTIME_PROT_READ: u32 = 1 << 0;
/// Indicates that the memory region should be writable.
pub const WASMTIME_PROT_WRITE: u32 = 1 << 1;
/// Indicates that the memory region should be executable.
pub const WASMTIME_PROT_EXEC: u32 = 1 << 2;

pub use WASMTIME_PROT_EXEC as PROT_EXEC;
pub use WASMTIME_PROT_READ as PROT_READ;
pub use WASMTIME_PROT_WRITE as PROT_WRITE;

/// Handler function for traps in Wasmtime passed to `wasmtime_init_traps`.
///
/// This function is invoked whenever a trap is caught by the system. For
/// example this would be invoked during a signal handler on Linux. This
/// function is passed a number of parameters indicating information about the
/// trap:
///
/// * `ip` - the instruction pointer at the time of the trap.
/// * `fp` - the frame pointer register's value at the time of the trap.
/// * `has_faulting_addr` - whether this trap is associated with an access
///   violation (e.g. a segfault) meaning memory was accessed when it shouldn't
///   be. If this is `true` then the next parameter is filled in.
/// * `faulting_addr` - if `has_faulting_addr` is true then this is the address
///   that was attempted to be accessed. Otherwise this value is not used.
///
/// If this function returns then the trap was not handled by Wasmtime. This
/// means that it's left up to the embedder how to deal with the trap/signal
/// depending on its default behavior. This could mean forwarding to a
/// non-Wasmtime handler, aborting the process, logging then crashing, etc. The
/// meaning of a trap that's not handled by Wasmtime depends on the context in
/// which the trap was generated.
///
/// When this function does not return it's because `wasmtime_longjmp` is
/// used to handle a Wasm-based trap.
pub type wasmtime_trap_handler_t =
    extern "C" fn(ip: usize, fp: usize, has_faulting_addr: bool, faulting_addr: usize);

extern "C" fn null_trap_handler(
    _ip: usize,
    _fp: usize,
    _has_faulting_addr: bool,
    _faulting_addr: usize,
) {
    // no-op
}

static mut g_handler: wasmtime_trap_handler_t = null_trap_handler;
/// Abstract pointer type used in the `wasmtime_memory_image_*` APIs which
/// is defined by the embedder.
pub enum wasmtime_memory_image {}

pub static mut WASMTIME_TLS: *mut u8 = 0 as *mut u8;

/// Creates a new virtual memory mapping of the `size` specified with
/// protection bits specified in `prot_flags`.
///
/// Memory can be lazily committed.
///
/// Stores the base pointer of the new mapping in `ret` on success.
///
/// Returns 0 on success and an error code on failure.
///
/// Similar to `mmap(0, size, prot_flags, MAP_PRIVATE, 0, -1)` on Linux.
#[no_mangle]
pub fn wasmtime_mmap_new(size: usize, prot_flags: u32, ret: &mut *mut u8) -> i32 {
    let prot_flags: ProtFlags = ProtFlags::from_bits(prot_flags).unwrap();
    let addr: MMResult<usize> = libos!(mmap(0, size, prot_flags, u32::MAX));
    if let Err(e) = addr {
        println!("[wasmtime] mmap_new failed: {:?}", e);
        return -1;
    }
    *ret = addr.unwrap() as *mut u8;
    0
}

/// Remaps the virtual memory starting at `addr` going for `size` bytes to
/// the protections specified with a new blank mapping.
///
/// This will unmap any prior mappings and decommit them. New mappings for
/// anonymous memory are used to replace these mappings and the new area
/// should have the protection specified by `prot_flags`.
///
/// Returns 0 on success and an error code on failure.
///
/// Similar to `mmap(addr, size, prot_flags, MAP_PRIVATE | MAP_FIXED, 0, -1)` on Linux.
#[no_mangle]
pub extern "C" fn wasmtime_mmap_remap(addr: *mut u8, size: usize, prot_flags: u32) -> i32 {
    let prot_flags: ProtFlags = ProtFlags::from_bits(prot_flags).unwrap();
    let addr: MMResult<usize> = libos!(mmap(addr as usize, size, prot_flags, addr as u32));
    if let Err(e) = addr {
        println!("[wasmtime] mmap_remap failed: {:?}", e);
        return -1;
    }
    0
}

/// Unmaps memory at the specified `ptr` for `size` bytes.
///
/// The memory should be discarded and decommitted and should generate a
/// segfault if accessed after this function call.
///
/// Returns 0 on success and an error code on failure.
///
/// Similar to `munmap` on Linux.
#[no_mangle]
pub extern "C" fn wasmtime_munmap(ptr: *mut u8, size: usize) -> i32 {
    let umap_data: &mut [u8] = unsafe { core::slice::from_raw_parts_mut(ptr, size) };
    if let Err(e) = libos!(munmap(umap_data, false)) {
        println!("[wasmtime] munmap failed: {:?}", e);
        return -1;
    }
    0
}

/// Configures the protections associated with a region of virtual memory
/// starting at `ptr` and going to `size`.
///
/// Returns 0 on success and an error code on failure.
///
/// Similar to `mprotect` on Linux.
#[no_mangle]
pub extern "C" fn wasmtime_mprotect(ptr: *mut u8, size: usize, prot_flags: u32) -> i32 {
    let prot_flags: ProtFlags = ProtFlags::from_bits(prot_flags).unwrap();
    if let Err(e) = libos!(mprotect(ptr as usize, size, prot_flags)) {
        println!("[wasmtime] mprotect failed: {:?}", e);
        return -1;
    }
    0
}

/// Returns the page size, in bytes, of the current system.
#[no_mangle]
pub extern "C" fn wasmtime_page_size() -> usize {
    4096
}

/// Used to setup a frame on the stack to longjmp back to in the future.
///
/// This function is used for handling traps in WebAssembly and is paried
/// with `wasmtime_longjmp`.
///
/// * `jmp_buf` - this argument is filled in with a pointer which if used
///   will be passed to `wasmtime_longjmp` later on by the runtime.
/// * `callback` - this callback should be invoked after `jmp_buf` is
///   configured.
/// * `payload` and `callee` - the two arguments to pass to `callback`.
///
/// Returns 0 if `wasmtime_longjmp` was used to return to this function.
/// Returns 1 if `wasmtime_longjmp` was not called and `callback` returned.
#[no_mangle]
pub extern "C" fn wasmtime_setjmp(
    jmp_buf: *mut *const u8,
    callback: extern "C" fn(*mut u8, *mut u8),
    payload: *mut u8,
    callee: *mut u8,
) -> i32 {
    unsafe {
        let mut buf = JumpBuf::new();
        if setjmp(&mut buf) != 0 {
            return 0;
        }
        *jmp_buf = &buf as *const JumpBuf as *const u8;
    }
    callback(payload, callee);
    1
}

/// Paired with `wasmtime_setjmp` this is used to jump back to the `setjmp`
/// point.
///
/// The argument here was originally passed to `wasmtime_setjmp` through its
/// out-param.
///
/// This function cannot return.
///
/// This function may be invoked from the `wasmtime_trap_handler_t`
/// configured by `wasmtime_init_traps`.
#[no_mangle]
pub extern "C" fn wasmtime_longjmp(jmp_buf: *const u8) {
    unsafe {
        longjmp(&*(jmp_buf as *const JumpBuf), 1);
    }
}

/// Initializes trap-handling logic for this platform.
///
/// Wasmtime's implementation of WebAssembly relies on the ability to catch
/// signals/traps/etc. For example divide-by-zero may raise a machine
/// exception. Out-of-bounds memory accesses may also raise a machine
/// exception. This function is used to initialize trap handling.
///
/// The `handler` provided is a function pointer to invoke whenever a trap
/// is encountered. The `handler` is invoked whenever a trap is caught by
/// the system.
///

const SA_SIGINFO: u32 = 0x00000008;
const SA_NODEFER: u32 = 0x40000000;
const SIGILL: i32 = 4;
const SIGSEGV: i32 = 11;
const SIGFPE: i32 = 8;

/// Returns 0 on success and an error code on failure.
fn handle_signal(sig: i32, info: *mut SigInfo, ctx: *mut Ucontext) {
    unsafe {
        if g_handler == null_trap_handler {
            return;
        }
        let ip = (*ctx).uc_mcontext.gregs[16];
        let fp = (*ctx).uc_mcontext.gregs[10];
        let has_faulting_addr = sig == SIGSEGV;
        let mut faulting_addr = 0;
        if has_faulting_addr {
            faulting_addr = (*info).siginfo.sifields.sigsys.call_addr as usize;
        }
        println!("handle_signal: sig: {}, ip: {}, fp: {}, has_faulting_addr: {}, faulting_addr: {}", sig, ip, fp, has_faulting_addr, faulting_addr);
        g_handler(ip, fp, has_faulting_addr, faulting_addr);
    }
    // signal(signo, SIG_DFL);
    println!("handle_signal: sig: {}, should signal", sig);
}


#[no_mangle]
pub extern "C" fn wasmtime_init_traps(handler: wasmtime_trap_handler_t) -> i32 {
    unsafe {
        g_handler = handler;
        let action = SigAction {
            sa_handler: handle_signal as usize,
            sa_flags: (SA_NODEFER | SA_SIGINFO) as usize,
            sa_mask: Sigset { sig: [0; 1] },
        };
        let rc = libos!(sigaction(SIGILL, &action, null()));
        if rc != 0 {
            return rc;
        }
    }
    0
}

/// Attempts to create a new in-memory image of the `ptr`/`len` combo which
/// can be mapped to virtual addresses in the future.
///
/// On successed the returned `wasmtime_memory_image` pointer is stored into `ret`.
/// This value stored can be `NULL` to indicate that an image cannot be
/// created but no failure occurred. The structure otherwise will later be
/// deallocated with `wasmtime_memory_image_free` and
/// `wasmtime_memory_image_map_at` will be used to map the image into new
/// regions of the address space.
///
/// The `ptr` and `len` arguments are only valid for this function call, if
/// the image needs to refer to them in the future then it must make a copy.
///
/// Both `ptr` and `len` are guaranteed to be page-aligned.
///
/// Returns 0 on success and an error code on failure. Note that storing
/// `NULL` into `ret` is not considered a failure, and failure is used to
/// indicate that something fatal has happened and Wasmtime will propagate
/// the error upwards.
#[no_mangle]
pub extern "C" fn wasmtime_memory_image_new(
    ptr: *const u8,
    len: usize,
    ret: &mut *mut wasmtime_memory_image,
) -> i32 {
    *ret = 0 as *mut wasmtime_memory_image;
    0
}

/// Maps the `image` provided to the virtual address at `addr` and `len`.
///
/// This semantically should make it such that `addr` and `len` looks the
/// same as the contents of what the memory image was first created with.
/// The mappings of `addr` should be private and changes do not reflect back
/// to `wasmtime_memory_image`.
///
/// In effect this is to create a copy-on-write mapping at `addr`/`len`
/// pointing back to the memory used by the image originally.
///
/// Note that the memory region will be unmapped with `wasmtime_munmap` in
/// the future.
///
/// Aborts the process on failure.
#[no_mangle]
pub extern "C" fn wasmtime_memory_image_map_at(
    image: *mut wasmtime_memory_image,
    addr: *mut u8,
    len: usize,
) -> i32 {
    0
}

/// Deallocates the provided `wasmtime_memory_image`.
///
/// Note that mappings created from this image are not guaranteed to be
/// deallocated and/or unmapped before this is called.
#[no_mangle]
pub extern "C" fn wasmtime_memory_image_free(image: *mut wasmtime_memory_image) {
    // no-op
}

/// Wasmtime requires a single pointer's space of TLS to be used at runtime,
/// and this function returns the current value of the TLS variable.
///
/// This value should default to `NULL`.
#[no_mangle]
pub extern "C" fn wasmtime_tls_get() -> *mut u8 {
    unsafe { WASMTIME_TLS }
}

/// Sets the current TLS value for Wasmtime to the provided value.
///
/// This value should be returned when later calling `wasmtime_tls_get`.
#[no_mangle]
pub extern "C" fn wasmtime_tls_set(ptr: *mut u8) {
    unsafe {
        WASMTIME_TLS = ptr;
    }
}
