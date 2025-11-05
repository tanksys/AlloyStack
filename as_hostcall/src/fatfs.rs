use alloc::string::String;
use thiserror_no_std::Error;

use crate::types::{Fd, OpenFlags, Size, Stat};

pub type FatfsOpenFunc = fn(&str, OpenFlags) -> FatfsResult<Fd>;
pub type FatfsWriteFunc = fn(Fd, &[u8]) -> FatfsResult<Size>;
pub type FatfsReadFunc = fn(Fd, &mut [u8]) -> FatfsResult<Size>;
pub type FatfsCloseFunc = fn(Fd) -> FatfsResult<()>;
pub type FatfsSeekFunc = fn(Fd, u32) -> FatfsResult<()>;
pub type FatfsStatFunc = fn(Fd) -> FatfsResult<Stat>;

pub type FatfsResult<T> = Result<T, FatfsError>;

#[derive(Debug, Error)]
#[repr(C)]
pub enum FatfsError {
    #[error("std::io::error, {0}")]
    HostIOErr(String),
    #[error("acquire std::sync::mutex::Mutex failed, {0}")]
    AcquireLockErr(String),
    #[error("bad input fd: {0}, do not exist")]
    BadInputFd(Fd),
    #[error("Unknown.")]
    Unknown,
}
