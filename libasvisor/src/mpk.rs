use std::sync::atomic::{AtomicUsize, Ordering};
use std::{arch::asm, ffi::c_void};

use anyhow::{anyhow, Ok};
use as_hostcall::mpk::LIBOS_PKEY;
use nix::{
    errno::Errno,
    libc::{size_t, syscall, SYS_pkey_alloc, SYS_pkey_mprotect},
};

use crate::{logger, utils};

fn _pkey_alloc() -> i32 {
    unsafe { syscall(SYS_pkey_alloc, 0, 0) as i32 }
}

pub fn must_init_all_pkeys() {
    for i in 0..16 {
        let pkey = _pkey_alloc();
        if pkey < 0 {
            panic!("pkey_alloc failed at {}", i);
        }
        if pkey == as_hostcall::mpk::LIBOS_PKEY {
            return;
        }
    }
}

// 0 -> default.
// 1..=13 -> user function.
// 14 -> DataBuffer.
// 15 -> backlist.
#[cfg(feature = "pkey_per_func")]
static PKEY_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn must_pkey_alloc() -> i32 {
    #[cfg(feature = "pkey_per_func")]
    {
        let previous_pkey = PKEY_COUNTER.fetch_add(1, Ordering::SeqCst);
        if previous_pkey == 13 {
            panic!("No more pkeys available");
        }
        (previous_pkey + 1) as i32
    }
    #[cfg(not(feature = "pkey_per_func"))]
    {
        const FUNC_PKEY: i32 = 1;
        FUNC_PKEY
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn pkey_mprotect(addr: *mut c_void, len: size_t, prot: i32, pkey: i32) -> anyhow::Result<()> {
    let ret: i64;
    unsafe {
        ret = syscall(SYS_pkey_mprotect, addr, len, prot, pkey);
    }
    match ret {
        0 => Ok(()),
        _ => Err(anyhow::Error::msg(format!(
            "mprotect failed, errno={}",
            Errno::last_raw()
        ))),
    }
}

pub fn pkey_read() -> u32 {
    // Reads the value of PKRU into EAX and clears EDX. ECX must be 0 when RDPKRU is executed; otherwise, a general-protection exception (#GP) occurs.
    let result: u32;
    // unsafe { asm!("rdpkru":  "=eax"(result) : "ecx"(0) : "edx": "volatile") }
    unsafe {
        asm!(
            "rdpkru",
            out("eax") result,
            in("ecx") 0,
            lateout("edx") _
        )
    };
    result
}

// #[inline]
// pub fn pkey_write(pkru: u32) {
//     // Writes the value of EAX into PKRU. ECX and EDX must be 0 when WRPKRU is executed;
//     // otherwise, a general-protection exception (#GP) occurs.
//     let eax = pkru;
//     let ecx = 0;
//     let edx = 0;

//     unsafe {
//         asm!(
//             "wrpkru",
//             in("eax") eax,
//             in("ecx") ecx,
//             in("edx") edx,
//         )
//     };
// }

pub fn grant_func_perm(pkru: u32, pkey: i32) -> anyhow::Result<u32> {
    if (0..16).contains(&pkey) {
        let mask: u32 = 0b11 << (2 * pkey);
        Ok(pkru & !mask)
    } else {
        Err(anyhow!("Invalid pkey: {}", pkey))
    }
}

pub fn set_libs_with_pkey(lib_abs_paths: &[&str], pkey: i32) -> Result<(), anyhow::Error> {
    let segments = utils::parse_memory_segments()?;
    for segment in segments {
        if let Some(path) = segment.clone().path {
            if lib_abs_paths.iter().any(|need| path.contains(need)) {
                pkey_mprotect(
                    segment.start_addr as *mut c_void,
                    segment.length,
                    segment.perm,
                    pkey,
                )?;

                logger::info!(
                    "{} (0x{:x}, 0x{:x}) set mpk success with pkey {}, right {:?}.",
                    segment.path.unwrap(),
                    segment.start_addr,
                    segment.start_addr + segment.length,
                    pkey,
                    segment.perm
                );
            }
        }
    }

    Ok(())
}

#[test]
fn test_mpk() {
    let pkey1 = pkey_alloc();
    println!("get pkey: {}", pkey1);
    let mm_layout = Layout::from_size_align(4096 * 10, 4096).unwrap();
    let mm_region = unsafe { alloc::alloc(mm_layout) };
    pkey_mprotect(
        mm_region.cast(),
        4096 * 10,
        libc::PROT_READ | libc::PROT_WRITE,
        pkey1,
    )
    .unwrap();
    let pkru = pkey_read();
    println!("pkru: {:b}", pkru);
    // 没有关闭 pkey1 的权限, 所以可以访问 group1 的内存
    let data = unsafe { slice::from_raw_parts_mut(mm_region, 4096 * 10) };
    data[0] = 100;
    println!("data: {}", data[0]);

    // 关闭读写权限.
    pkey_set(pkey1, 3).unwrap();
    let pkru = pkey_read();
    println!("pkru: {:b}", pkru);

    // 下一行预期会报错:
    // println!("data: {}", data[0]);
}
