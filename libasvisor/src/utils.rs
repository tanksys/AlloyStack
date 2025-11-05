use std::{fs, path::PathBuf, sync::Mutex};

use lazy_static::lazy_static;
use nix::libc;

pub const PAGE_SIZE: usize = 0x1000;

lazy_static! {
    static ref ID_KEEPER: Mutex<u64> = Mutex::new(1);
}

pub fn gen_new_id() -> u64 {
    let mut keeper = ID_KEEPER.lock().unwrap();
    let result = *keeper;
    *keeper += 1;
    result
}

#[macro_export]
macro_rules! round_up {
    ($x:expr) => {{
        use $crate::utils::PAGE_SIZE;
        ($x as usize + PAGE_SIZE - 1) & (!PAGE_SIZE + 1)
    }};
}

#[macro_export]
macro_rules! round_down {
    ($x:expr) => {{
        use $crate::utils::PAGE_SIZE;
        ($x as usize) & (!PAGE_SIZE + 1)
    }};
}

#[macro_export]
macro_rules! now_millis {
    () => {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
    };
}

#[macro_export]
macro_rules! now_microsec {
    () => {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_micros()
    };
}

#[test]
fn test_now_millis() {
    let now = now_millis!();
    assert!(now > 0, "now:{} not > 0", now)
}

#[test]
fn test_round_page() {
    assert_eq!(round_down!(0x1001), 0x1000);
    assert_eq!(round_down!(0x1fff), 0x1000);
    assert_eq!(round_down!(0x2000), 0x2000);

    assert_eq!(round_up!(0x2001), 0x3000);
    assert_eq!(round_up!(0x1fff), 0x2000);
    assert_eq!(round_up!(0x2000), 0x2000);
}

const REPOS_ROOT: &str = env!("CARGO_MANIFEST_DIR");
pub const PROFILE: &str = if cfg!(debug_assertions) {
    "debug"
} else {
    "release"
};

lazy_static! {
    pub static ref REPOS_ROOT_PATH: PathBuf =
        PathBuf::from(REPOS_ROOT).parent().unwrap().to_path_buf();
    pub static ref TARGET_DEBUG_PATH: PathBuf = REPOS_ROOT_PATH.join("target").join(PROFILE);
    pub static ref ISOL_CONFIG_PATH: PathBuf = REPOS_ROOT_PATH.join("isol_config");
}

#[test]
fn common_path_test() {
    use std::fs;

    fs::metadata(ISOL_CONFIG_PATH.join("base_config.json")).expect("base_config.json not found.");
}

use std::io::{self};

#[derive(Debug, Clone)]
pub struct MemorySegment {
    pub start_addr: usize,
    pub length: usize,
    pub perm: i32,
    pub path: Option<String>,
}

pub fn parse_memory_segments() -> io::Result<Vec<MemorySegment>> {
    let maps_str = fs::read_to_string("/proc/self/maps").unwrap();
    let lines = maps_str.lines().collect::<Vec<_>>();
    let mut segments = Vec::new();

    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();

        let path = if parts.len() == 6 {
            Some(parts[5].to_string())
        } else {
            None
        };

        let mut items = parts[0].split('-');
        let start_addr = usize::from_str_radix(items.next().unwrap(), 16).unwrap();
        let mut perm = 0i32;
        if parts[1].contains('r') {
            perm |= libc::PROT_READ
        }
        if parts[1].contains('w') {
            perm |= libc::PROT_WRITE
        }
        if parts[1].contains('x') {
            perm |= libc::PROT_EXEC
        }
        segments.push(MemorySegment {
            start_addr,
            length: usize::from_str_radix(items.next().unwrap(), 16).unwrap() - start_addr,
            perm,
            path,
        });
    }

    Ok(segments)
}
