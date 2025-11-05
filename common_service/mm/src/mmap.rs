extern crate alloc;

use core::alloc::Layout;

use alloc::borrow::ToOwned;
use libc::c_void;
use as_hostcall::{
    mm::{MMError, MMResult, ProtFlags},
    types::Fd,
};
use as_std::libos::libos;

const PAGE_SIZE: usize = 0x1000;

#[no_mangle]
pub fn libos_mmap(addr: usize, length: usize, prot: ProtFlags, fd: Fd) -> MMResult<usize> {
    if length % PAGE_SIZE > 0 {
        Err(MMError::InvaildArg(
            "length % PAGE_SIZE == 0".to_owned(),
            length,
        ))?
    }

    let layout = Layout::from_size_align(length, PAGE_SIZE)?;

    let mmap_addr = unsafe {
        let addr = {
            if addr == 0 {
                alloc::alloc::alloc(layout) as usize as *mut libc::c_void
            } else {
                addr as *mut libc::c_void
            }
        };
        if libc::munmap(addr, length) != 0 {
            Err(MMError::LibcErr("munmap failed".to_owned()))?
        }
        if libc::mmap(
            addr,
            length,
            trans_protflag(prot),
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            0,
            0,
        ) != addr
        {
            Err(MMError::LibcErr("mmap failed".to_owned()))?
        }

        addr as usize
    };

    if fd == u32::MAX {
        return Ok(mmap_addr);
    }

    let mm_region = unsafe { core::slice::from_raw_parts_mut(mmap_addr as *mut c_void, length) };
    libos!(register_file_backend(mm_region, fd))?;

    // println!("finish mmap, mmap_addr={}", mmap_addr);
    Ok(mmap_addr)
}

#[no_mangle]
pub fn libos_munmap(mem_region: &mut [u8], file_based: bool) -> MMResult<()> {
    if file_based {
        libos!(unregister_file_backend(mem_region.as_ptr() as usize))?;
    }

    let aligned_length = (mem_region.len() + PAGE_SIZE - 1) & (!PAGE_SIZE + 1);
    unsafe {
        if libc::mprotect(
            mem_region.as_mut_ptr() as usize as *mut libc::c_void,
            aligned_length,
            libc::PROT_READ | libc::PROT_WRITE,
        ) != 0
        {
            Err(MMError::LibcErr("mprotect failed".to_owned()))?
        };
        alloc::alloc::dealloc(
            mem_region.as_mut_ptr(),
            Layout::from_size_align(aligned_length, PAGE_SIZE)?,
        );
    };

    Ok(())
}

#[no_mangle]
pub fn libos_mprotect(addr: usize, length: usize, prot: ProtFlags) -> MMResult<()> {
    let aligned_length = (length + PAGE_SIZE - 1) & (!PAGE_SIZE + 1);
    unsafe {
        if libc::mprotect(
            addr as *mut libc::c_void,
            aligned_length,
            trans_protflag(prot),
        ) != 0
        {
            Err(MMError::LibcErr("mprotect failed".to_owned()))?
        };
    }
    Ok(())
}

pub fn trans_protflag(flags: ProtFlags) -> i32 {
    let mut result = Default::default();
    if flags.contains(ProtFlags::READ) {
        result |= libc::PROT_READ
    }
    if flags.contains(ProtFlags::WRITE) {
        result |= libc::PROT_WRITE
    }
    if flags.contains(ProtFlags::EXEC) {
        result |= libc::PROT_EXEC
    }

    result
}
