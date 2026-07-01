#![no_std]
#![feature(decl_macro)]

extern crate alloc;

use alloc::{format, string::String, vec::Vec};
use as_hostcall::{
    fdtab::FdtabError,
    mm::{MMError, ProtFlags},
    types::{OpenFlags, OpenMode, Stat},
};
use as_std::{libos::libos, println};
use core::{
    alloc::Layout,
    ffi::{c_char, c_void, CStr},
    mem, ptr,
    sync::atomic::{AtomicU64, Ordering},
};

const ENOENT: isize = 2;
const EBADF: isize = 9;
const ENOMEM: isize = 12;
const EFAULT: isize = 14;
const EINVAL: isize = 22;
const ENOTTY: isize = 25;
const ENOSYS: isize = 38;
const EEXIST: i32 = 17;
const EOVERFLOW: i32 = 75;
const EPROTO: i32 = 71;
const EMSGSIZE: i32 = 90;
const AT_FDCWD: isize = -100;
const AT_EMPTY_PATH: isize = 0x1000;

const SYS_READ: isize = 0;
const SYS_WRITE: isize = 1;
const SYS_OPEN: isize = 2;
const SYS_CLOSE: isize = 3;
const SYS_FSTAT: isize = 5;
const SYS_LSEEK: isize = 8;
const SYS_MMAP: isize = 9;
const SYS_MPROTECT: isize = 10;
const SYS_MUNMAP: isize = 11;
const SYS_BRK: isize = 12;
const SYS_IOCTL: isize = 16;
const SYS_READV: isize = 19;
const SYS_WRITEV: isize = 20;
const SYS_MREMAP: isize = 25;
const SYS_MADVISE: isize = 28;
const SYS_FCNTL: isize = 72;
const SYS_GETTID: isize = 186;
const SYS_FUTEX: isize = 202;
const SYS_OPENAT: isize = 257;
const SYS_NEWFSTATAT: isize = 262;
const SYS_GETRANDOM: isize = 318;

const O_ACCMODE: isize = 0o3;
const O_WRONLY: isize = 0o1;
const O_RDWR: isize = 0o2;
const O_CREAT: isize = 0o100;
const O_TRUNC: isize = 0o1000;
const O_APPEND: isize = 0o2000;

const F_GETFD: isize = 1;
const F_SETFD: isize = 2;
const F_GETFL: isize = 3;
const F_SETFL: isize = 4;

const PROT_READ: isize = 1;
const PROT_WRITE: isize = 2;
const PROT_EXEC: isize = 4;
const MREMAP_MAYMOVE: isize = 1;

static REPORTED_SYSCALLS: [AtomicU64; 8] = [
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
    AtomicU64::new(0),
];

const AS_BUFFER_MAGIC: u64 = 0x4153_4255_4646_4552;
const AS_BUFFER_FINGERPRINT: u64 = 0xd62e_ea91_45c8_8f31;

#[repr(C)]
struct AsBufferHeader {
    magic: u64,
    len: usize,
    capacity: usize,
}

#[repr(C)]
pub struct AsBuffer {
    pub data: *mut c_void,
    pub len: usize,
    pub capacity: usize,
    pub allocation: usize,
}

impl AsBuffer {
    const fn empty() -> Self {
        Self {
            data: ptr::null_mut(),
            len: 0,
            capacity: 0,
            allocation: 0,
        }
    }
}

fn as_buffer_layout(capacity: usize) -> Result<Layout, i32> {
    let size = mem::size_of::<AsBufferHeader>()
        .checked_add(capacity)
        .ok_or(-EOVERFLOW)?;
    Layout::from_size_align(size, mem::align_of::<AsBufferHeader>()).map_err(|_| -EINVAL as i32)
}

fn mm_error_code(error: &MMError) -> i32 {
    match error {
        MMError::NoMemory => -ENOMEM as i32,
        MMError::BufferSlotExists => -EEXIST,
        _ => -EINVAL as i32,
    }
}

unsafe fn buffer_slot<'a>(slot: *const c_char) -> Result<&'a str, i32> {
    if slot.is_null() {
        return Err(-EINVAL as i32);
    }
    let slot = CStr::from_ptr(slot).to_str().map_err(|_| -EINVAL as i32)?;
    if slot.is_empty() {
        return Err(-EINVAL as i32);
    }
    Ok(slot)
}

unsafe fn checked_header(buffer: *mut AsBuffer) -> Result<*mut AsBufferHeader, i32> {
    if buffer.is_null() {
        return Err(-EINVAL as i32);
    }
    let buffer_ref = &mut *buffer;
    if buffer_ref.allocation == 0 || buffer_ref.data.is_null() {
        return Err(-EINVAL as i32);
    }
    let header = buffer_ref.allocation as *mut AsBufferHeader;
    if (*header).magic != AS_BUFFER_MAGIC
        || (*header).capacity != buffer_ref.capacity
        || (header.add(1) as *mut c_void) != buffer_ref.data
    {
        return Err(-EINVAL as i32);
    }
    Ok(header)
}

#[no_mangle]
pub unsafe extern "C" fn as_buffer_alloc(capacity: usize, out: *mut AsBuffer) -> i32 {
    if out.is_null() {
        return -EINVAL as i32;
    }
    ptr::write(out, AsBuffer::empty());
    let layout = match as_buffer_layout(capacity) {
        Ok(layout) => layout,
        Err(error) => return error,
    };
    let allocation = match libos!(buffer_alloc_raw(layout)) {
        Ok(allocation) => allocation,
        Err(error) => return mm_error_code(&error),
    };
    let header = allocation as *mut AsBufferHeader;
    ptr::write(
        header,
        AsBufferHeader {
            magic: AS_BUFFER_MAGIC,
            len: 0,
            capacity,
        },
    );
    ptr::write(
        out,
        AsBuffer {
            data: header.add(1) as *mut c_void,
            len: 0,
            capacity,
            allocation,
        },
    );
    0
}

#[no_mangle]
pub unsafe extern "C" fn as_buffer_publish(
    slot: *const c_char,
    buffer: *mut AsBuffer,
    len: usize,
) -> i32 {
    let slot = match buffer_slot(slot) {
        Ok(slot) => slot,
        Err(error) => return error,
    };
    let header = match checked_header(buffer) {
        Ok(header) => header,
        Err(error) => return error,
    };
    if len > (*header).capacity {
        return -EMSGSIZE;
    }
    (*header).len = len;
    match libos!(buffer_register(
        slot,
        header as usize,
        AS_BUFFER_FINGERPRINT
    )) {
        Ok(()) => {
            ptr::write(buffer, AsBuffer::empty());
            0
        }
        Err(error) => mm_error_code(&error),
    }
}

#[no_mangle]
pub unsafe extern "C" fn as_buffer_take(slot: *const c_char, out: *mut AsBuffer) -> i32 {
    if out.is_null() {
        return -EINVAL as i32;
    }
    ptr::write(out, AsBuffer::empty());
    let slot = match buffer_slot(slot) {
        Ok(slot) => slot,
        Err(error) => return error,
    };
    let (allocation, fingerprint) = match libos!(access_buffer(slot)) {
        Some(metadata) => metadata,
        None => return -ENOENT as i32,
    };
    if fingerprint != AS_BUFFER_FINGERPRINT {
        let _ = libos!(buffer_register(slot, allocation, fingerprint));
        return -EPROTO;
    }
    let header = allocation as *mut AsBufferHeader;
    if (*header).magic != AS_BUFFER_MAGIC || (*header).len > (*header).capacity {
        let _ = libos!(buffer_register(slot, allocation, fingerprint));
        return -EPROTO;
    }
    ptr::write(
        out,
        AsBuffer {
            data: header.add(1) as *mut c_void,
            len: (*header).len,
            capacity: (*header).capacity,
            allocation,
        },
    );
    0
}

#[no_mangle]
pub unsafe extern "C" fn as_buffer_release(buffer: *mut AsBuffer) -> i32 {
    let header = match checked_header(buffer) {
        Ok(header) => header,
        Err(error) => return error,
    };
    let layout = match as_buffer_layout((*header).capacity) {
        Ok(layout) => layout,
        Err(error) => return error,
    };
    (*header).magic = 0;
    libos!(buffer_dealloc(header as usize, layout));
    ptr::write(buffer, AsBuffer::empty());
    0
}

#[repr(C)]
struct IoVec {
    base: *mut u8,
    len: usize,
}

fn neg_errno(errno: isize) -> isize {
    -errno
}

fn fd_error(error: FdtabError, opening: bool) -> isize {
    match error {
        FdtabError::BadInputFd(_, _) | FdtabError::NoExistFd(_) => {
            if opening {
                neg_errno(ENOENT)
            } else {
                neg_errno(EBADF)
            }
        }
        _ => neg_errno(EINVAL),
    }
}

fn open_file(path: *const c_char, flags: isize) -> isize {
    if path.is_null() {
        return neg_errno(EFAULT);
    }
    let path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(path) => path,
        Err(_) => return neg_errno(EINVAL),
    };

    let mode = match flags & O_ACCMODE {
        O_WRONLY => OpenMode::WR,
        O_RDWR => OpenMode::RDWR,
        _ => OpenMode::RD,
    };
    let mut open_flags = OpenFlags::empty();
    if flags & O_CREAT != 0 {
        open_flags |= OpenFlags::O_CREAT;
    }
    if flags & O_TRUNC != 0 {
        open_flags |= OpenFlags::O_TRUNC;
    }
    if flags & O_APPEND != 0 {
        open_flags |= OpenFlags::O_APPEND;
    }

    match libos!(open(path, open_flags, mode)) {
        Ok(fd) => fd as isize,
        Err(error) => fd_error(error, true),
    }
}

unsafe fn read_one(fd: u32, ptr: *mut u8, len: usize) -> isize {
    if ptr.is_null() && len != 0 {
        return neg_errno(EFAULT);
    }
    let buffer = core::slice::from_raw_parts_mut(ptr, len);
    match libos!(read(fd, buffer)) {
        Ok(size) => size as isize,
        Err(error) => fd_error(error, false),
    }
}

unsafe fn write_one(fd: u32, ptr: *const u8, len: usize) -> isize {
    if ptr.is_null() && len != 0 {
        return neg_errno(EFAULT);
    }
    let buffer = core::slice::from_raw_parts(ptr, len);
    match libos!(write(fd, buffer)) {
        Ok(size) => size as isize,
        Err(error) => fd_error(error, false),
    }
}

unsafe fn read_vectored(fd: u32, iov: *mut IoVec, count: usize) -> isize {
    if iov.is_null() && count != 0 {
        return neg_errno(EFAULT);
    }
    let mut total = 0isize;
    for item in core::slice::from_raw_parts_mut(iov, count) {
        let read = read_one(fd, item.base, item.len);
        if read < 0 {
            return if total == 0 { read } else { total };
        }
        total += read;
        if read as usize != item.len {
            break;
        }
    }
    total
}

unsafe fn write_vectored(fd: u32, iov: *const IoVec, count: usize) -> isize {
    if iov.is_null() && count != 0 {
        return neg_errno(EFAULT);
    }
    let mut total = 0isize;
    for item in core::slice::from_raw_parts(iov, count) {
        let written = write_one(fd, item.base, item.len);
        if written < 0 {
            return if total == 0 { written } else { total };
        }
        total += written;
        if written as usize != item.len {
            break;
        }
    }
    total
}

fn report_unsupported(number: isize) -> isize {
    let should_report = if (0..512).contains(&number) {
        let bit = 1u64 << (number as usize & 63);
        REPORTED_SYSCALLS[number as usize / 64].fetch_or(bit, Ordering::Relaxed) & bit == 0
    } else {
        true
    };
    if should_report {
        println!("as_musl: unsupported syscall {}", number);
    }
    neg_errno(ENOSYS)
}

#[no_mangle]
pub unsafe extern "C" fn alloy_syscall(
    number: isize,
    a1: isize,
    a2: isize,
    a3: isize,
    a4: isize,
    a5: isize,
    a6: isize,
) -> isize {
    match number {
        SYS_READ => read_one(a1 as u32, a2 as *mut u8, a3 as usize),
        SYS_WRITE => write_one(a1 as u32, a2 as *const u8, a3 as usize),
        SYS_READV => read_vectored(a1 as u32, a2 as *mut IoVec, a3 as usize),
        SYS_WRITEV => write_vectored(a1 as u32, a2 as *const IoVec, a3 as usize),
        SYS_OPEN => open_file(a1 as *const c_char, a2),
        SYS_OPENAT if a1 == AT_FDCWD => open_file(a2 as *const c_char, a3),
        SYS_OPENAT => neg_errno(EINVAL),
        SYS_CLOSE => match libos!(close(a1 as u32)) {
            Ok(()) => 0,
            Err(error) => fd_error(error, false),
        },
        SYS_LSEEK => match libos!(lseek64(a1 as u32, a2 as i64, a3 as i32)) {
            Ok(offset) => offset as isize,
            Err(error) => fd_error(error, false),
        },
        SYS_FSTAT => {
            if (a2 as *mut Stat).is_null() {
                return neg_errno(EFAULT);
            }
            match libos!(stat(a1 as u32)) {
                Ok(stat) => {
                    core::ptr::write(a2 as *mut Stat, stat);
                    0
                }
                Err(error) => fd_error(error, false),
            }
        }
        SYS_NEWFSTATAT if a1 >= 0 && a4 & AT_EMPTY_PATH != 0 => {
            alloy_syscall(SYS_FSTAT, a1, a3, 0, 0, 0, 0)
        }
        SYS_MMAP => {
            let mut prot = ProtFlags::empty();
            if a3 & PROT_READ != 0 {
                prot |= ProtFlags::READ;
            }
            if a3 & PROT_WRITE != 0 {
                prot |= ProtFlags::WRITE;
            }
            if a3 & PROT_EXEC != 0 {
                prot |= ProtFlags::EXEC;
            }
            let fd = if a5 < 0 { u32::MAX } else { a5 as u32 };
            match libos!(mmap(a1 as usize, a2 as usize, prot, fd)) {
                Ok(addr) => addr as isize,
                Err(_) => neg_errno(ENOMEM),
            }
        }
        SYS_MUNMAP => {
            if a1 == 0 {
                return neg_errno(EINVAL);
            }
            let region = core::slice::from_raw_parts_mut(a1 as *mut u8, a2 as usize);
            match libos!(munmap(region, false)) {
                Ok(()) => 0,
                Err(_) => neg_errno(EINVAL),
            }
        }
        SYS_MPROTECT => {
            let mut prot = ProtFlags::empty();
            if a3 & PROT_READ != 0 {
                prot |= ProtFlags::READ;
            }
            if a3 & PROT_WRITE != 0 {
                prot |= ProtFlags::WRITE;
            }
            if a3 & PROT_EXEC != 0 {
                prot |= ProtFlags::EXEC;
            }
            match libos!(mprotect(a1 as usize, a2 as usize, prot)) {
                Ok(()) => 0,
                Err(_) => neg_errno(EINVAL),
            }
        }
        SYS_MREMAP => {
            if a1 == 0 || a2 == 0 || a3 == 0 || a4 != MREMAP_MAYMOVE || a5 != 0 {
                return neg_errno(EINVAL);
            }
            let prot = ProtFlags::READ | ProtFlags::WRITE;
            let new_addr = match libos!(mmap(0, a3 as usize, prot, u32::MAX)) {
                Ok(addr) => addr,
                Err(_) => return neg_errno(ENOMEM),
            };
            core::ptr::copy_nonoverlapping(
                a1 as *const u8,
                new_addr as *mut u8,
                core::cmp::min(a2 as usize, a3 as usize),
            );
            let old_region = core::slice::from_raw_parts_mut(a1 as *mut u8, a2 as usize);
            if libos!(munmap(old_region, false)).is_err() {
                let new_region = core::slice::from_raw_parts_mut(new_addr as *mut u8, a3 as usize);
                let _ = libos!(munmap(new_region, false));
                return neg_errno(EINVAL);
            }
            new_addr as isize
        }
        SYS_MADVISE => 0,
        SYS_BRK => neg_errno(ENOMEM),
        SYS_IOCTL => neg_errno(ENOTTY),
        SYS_FCNTL => match a2 {
            F_GETFD | F_GETFL => 0,
            F_SETFD | F_SETFL => 0,
            _ => neg_errno(EINVAL),
        },
        SYS_FUTEX => libos!(futex(
            a1 as *mut i32,
            a2 as i32,
            a3 as i32,
            a4 as *const u8,
            a5 as *mut i32,
            a6 as i32
        )),
        SYS_GETTID => libos!(gettid()),
        SYS_GETRANDOM => libos!(getrandom(a1 as *mut u8, a2 as usize, a3 as u32)),
        _ => report_unsupported(number),
    }
}

extern "C" {
    fn alloy_musl_thread_init();
    fn alloy_musl_flush() -> i32;
}

pub unsafe fn run_c_main(
    function_name: &str,
    c_main: unsafe extern "C" fn(i32, *mut *mut c_char) -> i32,
) -> FaaSFuncResult<()> {
    alloy_musl_thread_init();

    let mut storage: Vec<Vec<u8>> = Vec::new();
    let mut program = function_name.as_bytes().to_vec();
    program.push(0);
    storage.push(program);

    for (key, value) in as_std::args::all() {
        if key.as_bytes().contains(&0) || value.as_bytes().contains(&0) {
            return Err(String::from("C argv contains NUL").into());
        }
        let mut arg = format!("--{}={}", key, value).into_bytes();
        arg.push(0);
        storage.push(arg);
    }

    let mut argv: Vec<*mut c_char> = storage
        .iter_mut()
        .map(|value| value.as_mut_ptr() as *mut c_char)
        .collect();
    argv.push(core::ptr::null_mut());

    let code = c_main((argv.len() - 1) as i32, argv.as_mut_ptr());
    let flush_code = alloy_musl_flush();
    if code != 0 {
        Err(format!("C main returned {}", code).into())
    } else if flush_code != 0 {
        Err(String::from("musl fflush failed").into())
    } else {
        Ok(().into())
    }
}

pub macro entry($c_main:ident) {
    extern "C" {
        fn $c_main(argc: i32, argv: *mut *mut core::ffi::c_char) -> i32;
    }

    #[no_mangle]
    pub fn main() -> $crate::FaaSFuncResult<()> {
        unsafe { $crate::run_c_main(env!("CARGO_PKG_NAME"), $c_main) }
    }
}

pub use as_std::agent::FaaSFuncResult;
