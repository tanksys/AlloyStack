#![allow(improper_ctypes_definitions)]

use alloc::string::String;

use crate::{HostCallID, IsolationContext};

use bitflags::bitflags;

pub type IsolationID = u64;
pub type ServiceName = String;
pub type SymbolName = String;
pub type HostCallResult = Result<(), HostCallError>;

pub struct NetdevName {
    pub name: String, // like "tap-7a323b"
    pub subnet: core::net::Ipv4Addr,
    pub mask: u8,
}

#[derive(Debug)]
pub enum HostCallError {
    HasBeenSet,
}

// msvisor
pub type FindHostCallFunc = unsafe extern "C" fn(IsolationID, HostCallID) -> usize;

// service init
pub type SetHandlerFunc = unsafe extern "C" fn(&IsolationContext) -> HostCallResult;
pub type GetHandlerFunc = unsafe extern "C" fn() -> usize;
pub type PanicHandlerFunc = unsafe extern "C" fn() -> !;

// service drop
pub type DropHandlerFunc = unsafe fn();

// app main
pub type RustMainFunc = unsafe fn() -> u64;

// fdtab
bitflags! {
    pub struct OpenFlags: u32 {
       const O_APPEND = 1;
       const O_CREAT = 2;
    }

    #[derive(PartialEq, Clone)]
    pub struct OpenMode: u32 {
        const RD = 1;
        const WR = 2;
        const RDWR = 3;
    }
}
pub type Fd = u32;
pub type Size = usize;

// time for stat
#[derive(Debug)]
pub struct TimeSpec {
    /// Whole seconds part of the timespec.
    pub tv_sec: core::ffi::c_longlong,
    /// Nanoseconds part of the timespec, complementing `tv_sec`.
    pub tv_nsec: core::ffi::c_long,
}

// file stat
#[derive(Debug)]
pub struct Stat {
    // pub st_size: Size,
    /// Device identifier.
    pub st_dev: u64,
    /// Inode number.
    pub st_ino: u64,
    /// Number of hard links.
    pub st_nlink: u64,
    /// File mode and permissions.
    pub st_mode: core::ffi::c_uint,
    /// User ID of owner.
    pub st_uid: core::ffi::c_uint,
    /// Group ID of owner.
    pub st_gid: core::ffi::c_uint,
    /// Padding to maintain alignment.
    pub __pad0: core::ffi::c_uint,
    /// Device ID (if special file).
    pub st_rdev: u64,
    /// Total size, in bytes.
    pub st_size: Size,
    /// Block size for filesystem I/O.
    pub st_blksize: core::ffi::c_long,
    /// Number of 512B blocks allocated.
    pub st_blocks: i64,
    /// Time of last access.
    pub st_atime: TimeSpec,
    /// Time of last modification.
    pub st_mtime: TimeSpec,
    /// Time of last status change.
    pub st_ctime: TimeSpec,
    /// Unused space, reserved for future use.
    pub __unused: [core::ffi::c_long; 3usize],
}

// dir entry
#[derive(Debug)]
pub struct DirEntry {
    pub dir_path: String,
    pub entry_name: String,
    pub entry_type: u32,
}

// stdio
pub type HostStdioFunc = fn(&[u8]) -> Size;

// socket
pub type SockFd = u32;

// time
pub type GetTimeFunc = fn() -> Result<u128, String>;
pub type NanoSleepFunc = fn(u64, u64);

// isol_info
pub type MetricFunc = fn(IsolationID, MetricEvent) -> Result<(), ()>;
pub type FsImageFunc = fn(IsolationID) -> Option<String>;
pub type SpawnFaultThreadFunc = fn(IsolationID) -> Result<(), String>;

#[derive(Debug)]
pub enum MetricEvent {
    // IsolationEvent
    IsolBegin,
    LoadService,
    IsolEnd,

    // SvcEvent
    SvcInit,
    SvcRun,
    SvcEnd,

    // MesureEvent
    Mem,
}

pub trait Transmutor {
    fn find_host_call() -> FindHostCallFunc;
    fn host_panic_handler() -> PanicHandlerFunc;
}
