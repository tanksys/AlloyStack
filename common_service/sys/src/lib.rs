#![no_std]

pub use as_std;
use core::ffi::c_void;

unsafe fn syscall_result(ret: libc::c_long) -> isize {
    if ret == -1 {
        -(*libc::__errno_location() as isize)
    } else {
        ret as isize
    }
}

#[no_mangle]
pub unsafe extern "C" fn host_futex(
    uaddr: *mut i32,
    op: i32,
    val: i32,
    timeout: *const u8,
    uaddr2: *mut i32,
    val3: i32,
) -> isize {
    syscall_result(libc::syscall(
        libc::SYS_futex,
        uaddr,
        op,
        val,
        timeout as *const c_void,
        uaddr2,
        val3,
    ))
}

#[no_mangle]
pub extern "C" fn host_gettid() -> isize {
    unsafe { libc::syscall(libc::SYS_gettid) as isize }
}

#[no_mangle]
pub unsafe extern "C" fn host_getrandom(buf: *mut u8, len: usize, flags: u32) -> isize {
    syscall_result(libc::syscall(libc::SYS_getrandom, buf, len, flags))
}
