use core::{
    fmt::Display,
    net::{Ipv4Addr, SocketAddrV4},
};

use alloc::vec::Vec;
use as_hostcall::{fdtab::FdtabError, socket::SmoltcpError, types::Fd};

use crate::{
    io::{Read, Write},
    libos::libos,
};

pub struct TcpStream {
    raw_fd: Fd,
}

impl TcpStream {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self, FdtabError> {
        // println!("connect to {}", addr);

        let sockaddrv4 = match addr.to_socket_addrs()?.inner {
            core::net::SocketAddr::V4(addr) => addr,
            core::net::SocketAddr::V6(_) => todo!(),
        };
        let raw_fd = libos!(connect(sockaddrv4))?;
        let stream = Self { raw_fd };

        Ok(stream)
    }

    // fn write_str(&mut self, s: &str) -> core::fmt::Result {
    //     self.write_all(s.as_bytes()).map_err(|_| core::fmt::Error)
    // }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize, FdtabError> {
        libos!(write(self.raw_fd, buf))
    }
}

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, FdtabError> {
        libos!(read(self.raw_fd, buf))
    }
}

impl Drop for TcpStream {
    fn drop(&mut self) {
        libos!(close(self.raw_fd)).expect("close tcp error?")
    }
}

pub struct Incoming<'a> {
    listener: &'a TcpListener,
}

impl<'a> Iterator for Incoming<'a> {
    type Item = TcpStream;

    fn next(&mut self) -> Option<Self::Item> {
        libos!(accept(self.listener.raw_fd))
            .ok()
            .map(|raw_fd| Self::Item { raw_fd })
    }
}

pub struct TcpListener {
    raw_fd: Fd,
}

impl TcpListener {
    pub fn bind<A: ToSocketAddrs>(url: A) -> Result<Self, FdtabError> {
        let addr: SocketAddr = url.to_socket_addrs().unwrap();
        let sockaddrv4 = match addr.inner {
            core::net::SocketAddr::V4(addr) => addr,
            core::net::SocketAddr::V6(_) => todo!(),
        };
        libos!(bind(sockaddrv4)).map(|raw_fd| Self { raw_fd })
    }

    pub fn incoming(&self) -> Incoming {
        Incoming { listener: self }
    }
}

impl Drop for TcpListener {
    fn drop(&mut self) {
        libos!(close(self.raw_fd)).expect("close failed.")
    }
}

pub struct SocketAddr {
    inner: core::net::SocketAddr,
}

impl Default for SocketAddr {
    fn default() -> Self {
        Self {
            inner: core::net::SocketAddr::from(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0)),
        }
    }
}

impl Display for SocketAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.inner.fmt(f)
    }
}

pub trait ToSocketAddrs {
    fn to_socket_addrs(&self) -> Result<SocketAddr, SmoltcpError>;
}

impl ToSocketAddrs for &str {
    fn to_socket_addrs(&self) -> Result<SocketAddr, SmoltcpError> {
        let mut sockaddr = SocketAddr::default();

        let (ip, port) = {
            let target_tuple: Vec<&str> = self.split(':').collect();
            let host_str = target_tuple[0];
            let addr = if let Ok(addr) = host_str.parse() {
                addr
            } else {
                libos!(addrinfo(host_str)).expect("wrong tcp domain")
            };

            let port = match target_tuple.get(1) {
                Some(port) => port.parse::<u16>().expect("wrong url"),
                None => 80,
            };

            (addr, port)
        };

        sockaddr.inner = core::net::SocketAddr::from(SocketAddrV4::new(ip, port));

        Ok(sockaddr)
    }
}
