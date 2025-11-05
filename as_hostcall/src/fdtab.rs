use core::net::SocketAddrV4;

use alloc::{string::String, vec::Vec};
use thiserror_no_std::Error;

use crate::{
    fatfs::FatfsError,
    socket::SmoltcpError,
    types::{Fd, OpenFlags, OpenMode, Size, SockFd, Stat, DirEntry},
};

pub type OpenFunc = fn(&str, OpenFlags, OpenMode) -> FdtabResult<Fd>;
pub type WriteFunc = fn(Fd, &[u8]) -> FdtabResult<Size>;
pub type ReadFunc = fn(Fd, &mut [u8]) -> FdtabResult<Size>;
pub type CloseFunc = fn(Fd) -> FdtabResult<()>;
pub type LseekFunc = fn(Fd, u32) -> FdtabResult<()>;
pub type StatFunc = fn(Fd) -> FdtabResult<Stat>;
pub type ReadDirFunc = fn(&str) -> FdtabResult<Vec<DirEntry>>;
pub type ConnectFunc = fn(SocketAddrV4) -> FdtabResult<Fd>;
pub type BindFunc = fn(SocketAddrV4) -> FdtabResult<Fd>;
pub type AcceptFunc = fn(SockFd) -> FdtabResult<SockFd>;

pub type FdtabResult<T> = Result<T, FdtabError>;

#[derive(Debug, Error)]
pub enum FdtabError {
    #[error("bad input fd, expect {0}, found {1}")]
    BadInputFd(String, Fd),
    #[error("file not found, fd={0}")]
    NoExistFd(Fd),
    #[error("missing read permission, fd={0}")]
    NoReadPerm(Fd),
    #[error("missing write permission, fd={0}")]
    NoWritePerm(Fd),
    #[error("fatfs error: {0}")]
    FatfsError(#[from] FatfsError),
    #[error("smoltcp error: {0}")]
    SocketError(#[from] SmoltcpError),
    #[error("undefine {op} to {fd_type}, fd={fd}")]
    UndefinedOperation { op: String, fd: Fd, fd_type: String },
    #[error("ruxfs error: {0}")]
    RuxfsError(String),
}
