#![no_std]

use nix::{
    libc::{timespec, CLOCK_REALTIME},
    time::clock_gettime,
};

#[no_mangle]
pub extern "C" fn get_time() -> u64 {
    let Ok(r) = clock_gettime(CLOCK_REALTIME.into()) else {
        return u64::MAX;
    };
    let Some(nanos) = (r.tv_sec() as u64)
        .checked_mul(1_000_000_000)
        .and_then(|seconds| seconds.checked_add(r.tv_nsec() as u64))
    else {
        return u64::MAX;
    };
    nanos
}

#[no_mangle]
pub fn host_nanosleep(sec: u64, nsec: u64) {
    let mut ts = timespec {
        tv_sec: sec as i64,
        tv_nsec: nsec as i64,
    };
    let ts_ptr = &mut ts as *mut _;

    unsafe { nix::libc::nanosleep(ts_ptr, ts_ptr) };
}

#[test]
fn get_time_test() {
    let t = get_time();
    assert_ne!(t, u64::MAX, "clock_gettime failed");
    assert!(t > 1_577_836_800 * 1_000_000_000, "error time: {}", t);
}

#[test]
fn test_nanosleep() {
    host_nanosleep(1, 0)
}
