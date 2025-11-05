use core::arch::asm;

use as_hostcall::mpk::LIBOS_PKEY;

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

pub fn grant_libos_perm(pkru: u32) -> u32 {
    let mask: u32 = 0b11 << (2 * LIBOS_PKEY);
    pkru & !mask
}

pub fn drop_libos_perm(pkru: u32) -> u32 {
    let mask: u32 = 0b11 << (2 * LIBOS_PKEY);
    pkru | mask
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

// #[inline]
// pub fn pkey_set(pkey: i32, rights: u32) -> Result<(), &'static str> {
//     if (0..16).contains(&pkey) {
//         let mask: u32 = 0b11 << (2 * pkey);
//         let mut pkru = pkey_read();
//         pkru = (pkru & !mask) | (rights << (2 * pkey));
//         pkey_write(pkru);
//         Ok(())
//     } else {
//         Err("Invalid PKEY")
//     }
// }
