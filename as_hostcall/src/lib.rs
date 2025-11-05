#![no_std]
#![feature(ip_in_core)]
#![feature(decl_macro)]
#![feature(error_in_core)]

extern crate alloc;

pub mod err;
#[cfg(feature = "fatfs")]
pub mod fatfs;
#[cfg(feature = "fdtab")]
pub mod fdtab;
#[cfg(feature = "mm")]
pub mod mm;
#[cfg(feature = "mmap_file_backend")]
pub mod mmap_file_backend;
#[cfg(feature = "enable_mpk")]
pub mod mpk;
#[cfg(feature = "signal")]
pub mod signal;
#[cfg(feature = "socket")]
pub mod socket;
pub mod types;

use alloc::{borrow::ToOwned, string::String};
use types::{IsolationID, ServiceName};

use derive_more::Display;

#[derive(Debug, Display)]
#[repr(C)]
pub enum CommonHostCall {
    #[display(fmt = "metric")]
    Metric,
    #[display(fmt = "fs_image")]
    FsImage,
    #[display(fmt = "spawn_fault_thread")]
    SpawnFaultThread,

    #[display(fmt = "write")]
    Write,
    #[display(fmt = "read")]
    Read,
    #[display(fmt = "open")]
    Open,
    #[display(fmt = "close")]
    Close,
    #[display(fmt = "lseek")]
    Lseek,
    #[display(fmt = "stat")]
    Stat,
    #[display(fmt = "readdir")]
    ReadDir,
    #[display(fmt = "connect")]
    Connect,
    #[display(fmt = "socket")]
    Socket,
    #[display(fmt = "bind")]
    Bind,
    #[display(fmt = "accept")]
    Accept,

    #[display(fmt = "host_stdout")]
    Stdout,

    #[display(fmt = "fatfs_open")]
    FatfsOpen,
    #[display(fmt = "fatfs_write")]
    FatfsWrite,
    #[display(fmt = "fatfs_read")]
    FatfsRead,
    #[display(fmt = "fatfs_close")]
    FatfsClose,
    #[display(fmt = "fatfs_seek")]
    FatfsSeek,
    #[display(fmt = "fatfs_stat")]
    FatfsStat,

    #[display(fmt = "addrinfo")]
    SmoltcpAddrInfo,
    #[display(fmt = "smol_connect")]
    SmoltcpConnect,
    #[display(fmt = "smol_send")]
    SmoltcpSend,
    #[display(fmt = "smol_recv")]
    SmoltcpRecv,
    #[display(fmt = "smol_bind")]
    SmoltcpBind,
    #[display(fmt = "smol_accept")]
    SmoltcpAccept,
    #[display(fmt = "smol_close")]
    SmoltcpClose,

    #[display(fmt = "buffer_alloc")]
    BufferAlloc,
    #[display(fmt = "access_buffer")]
    AccessBuffer,
    #[display(fmt = "buffer_dealloc")]
    BufferDealloc,
    #[display(fmt = "libos_mmap")]
    Mmap,
    #[display(fmt = "libos_munmap")]
    Munmap,
    #[display(fmt = "libos_mprotect")]
    Mprotect,

    #[display(fmt = "register_file_backend")]
    RegisterFileBackend,
    #[display(fmt = "unregister_file_backend")]
    UnregisterFileBackend,
    #[display(fmt = "file_page_fault_handler")]
    FilePageFaultHandler,

    #[display(fmt = "get_time")]
    GetTime,
    #[display(fmt = "host_nanosleep")]
    NanoSleep,

    #[display(fmt = "libos_sigaction")]
    SigAction,
}

#[derive(Debug, Display)]
#[repr(C)]
pub enum HostCallID {
    Common(CommonHostCall),
    Custom(String),
}
impl HostCallID {
    pub fn belong_to(&self) -> ServiceName {
        match self {
            Self::Common(common) => match common {
                CommonHostCall::Metric
                | CommonHostCall::FsImage
                | CommonHostCall::SpawnFaultThread => "".to_owned(),

                CommonHostCall::Write
                | CommonHostCall::Open
                | CommonHostCall::Read
                | CommonHostCall::Close
                | CommonHostCall::Lseek
                | CommonHostCall::Stat
                | CommonHostCall::ReadDir
                | CommonHostCall::Connect
                | CommonHostCall::Socket
                | CommonHostCall::Bind
                | CommonHostCall::Accept => "fdtab".to_owned(),

                CommonHostCall::Stdout => "stdio".to_owned(),

                CommonHostCall::FatfsOpen
                | CommonHostCall::FatfsWrite
                | CommonHostCall::FatfsRead
                | CommonHostCall::FatfsClose
                | CommonHostCall::FatfsSeek
                | CommonHostCall::FatfsStat => "fatfs".to_owned(),

                CommonHostCall::SmoltcpAddrInfo
                | CommonHostCall::SmoltcpConnect
                | CommonHostCall::SmoltcpSend
                | CommonHostCall::SmoltcpRecv
                | CommonHostCall::SmoltcpBind
                | CommonHostCall::SmoltcpAccept
                | CommonHostCall::SmoltcpClose => "socket".to_owned(),

                CommonHostCall::BufferAlloc
                | CommonHostCall::AccessBuffer
                | CommonHostCall::BufferDealloc
                | CommonHostCall::Mmap
                | CommonHostCall::Munmap
                | CommonHostCall::Mprotect => "mm".to_owned(),

                CommonHostCall::RegisterFileBackend
                | CommonHostCall::FilePageFaultHandler
                | CommonHostCall::UnregisterFileBackend => "mmap_file_backend".to_owned(),

                CommonHostCall::SigAction => "signal".to_owned(),

                CommonHostCall::GetTime | CommonHostCall::NanoSleep => "time".to_owned(),
            },
            HostCallID::Custom(_) => todo!(),
        }
    }
}

#[test]
fn format_hostcall_id() {
    use crate::alloc::string::ToString;

    let result = CommonHostCall::Write;
    assert!(
        result.to_string().eq("write"),
        "actual format result is {}",
        result
    )
}

#[derive(Clone, Default)]
#[repr(C)]
pub struct IsolationContext {
    pub isol_id: IsolationID,
    pub find_handler: usize,
    pub panic_handler: usize,
    pub heap_range: (usize, usize),
}

impl IsolationContext {
    pub const fn uninit() -> Self {
        Self {
            isol_id: 0,
            find_handler: 0,
            panic_handler: 0,
            heap_range: (0, 0),
        }
    }
}

pub const SERVICE_HEAP_SIZE: usize = 4 * 1024 * 1024 * 1024;
pub const SERVICE_STACK_SIZE: usize = 8 * 1024 * 1024;

pub trait Verify {
    fn __fingerprint() -> u64;
}

impl Verify for () {
    fn __fingerprint() -> u64 {
        let v: i64 = -2542357861231615084;
        unsafe { *(&v as *const _ as usize as *const u64) }
    }
}

impl Verify for i32 {
    fn __fingerprint() -> u64 {
        let v: i64 = 0x43c29777932e38aei64;
        unsafe { *(&v as *const _ as usize as *const u64) }
    }
}
