use core::net::{Ipv4Addr, SocketAddrV4};

use alloc::string::String;
use thiserror_no_std::Error;

use crate::types::{Size, SockFd};

pub type SmoltcpAddrInfoFunc = fn(&str) -> SmoltcpResult<Ipv4Addr>;
pub type SmoltcpConnectFunc = fn(SocketAddrV4) -> SmoltcpResult<SockFd>;
pub type SmoltcpSendFunc = fn(SockFd, &[u8]) -> SmoltcpResult<()>;
pub type SmoltcpRecvFunc = fn(SockFd, &mut [u8]) -> SmoltcpResult<Size>;
pub type SmoltcpBindFunc = fn(SocketAddrV4) -> SmoltcpResult<SockFd>;
pub type SmoltcpAcceptFunc = fn(SockFd) -> SmoltcpResult<SockFd>;
pub type SmoltcpCloseFunc = fn(SockFd) -> SmoltcpResult<()>;

pub type SmoltcpResult<T> = Result<T, SmoltcpError>;

#[derive(Error, Debug)]
pub enum SmoltcpError {
    #[error("acquire std::sync::mutex::Mutex failed, name={0}, reason={1}")]
    AcquireLockErr(String, String),
    #[error("smoltcp error, {0}")]
    SmoltcpErr(String),
    #[error("std::io::error, {0}")]
    HostIOErr(String),
    #[error("unknown reason, expect have query result")]
    DNSQueryFailed,
    #[error("wrong tcp state, expect={0}, found={1}")]
    BadTCPState(String, String),
    #[error("unknown reason, expect have local endpoint")]
    NoLocalEndpoint,
}
