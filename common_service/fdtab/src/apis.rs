use core::net::SocketAddrV4;

use alloc::borrow::ToOwned;
use as_hostcall::{
    fdtab::{FdtabError, FdtabResult},
    types::{Fd, OpenFlags, OpenMode, Size, SockFd, Stat},
};
use as_std::libos::libos;

use crate::{DataSource, File, FD_TABLE};

#[no_mangle]
pub fn read(fd: Fd, buf: &mut [u8]) -> FdtabResult<Size> {
    if let 0..=2 = fd {
        Err(FdtabError::BadInputFd(">2".to_owned(), fd))?
    }

    FD_TABLE.with_file(fd, |file| -> FdtabResult<Size> {
        let file = file.ok_or(FdtabError::NoExistFd(fd))?;

        if !file.can_read() {
            Err(FdtabError::NoReadPerm(fd))?
        }

        Ok(match file.src {
            DataSource::FatFS(raw_fd) => libos!(fatfs_read(raw_fd, buf))?,
            DataSource::Net(socket) => libos!(recv(socket, buf))?,
        })
    })
}

#[no_mangle]
pub fn write(fd: Fd, buf: &[u8]) -> FdtabResult<Size> {
    match fd {
        0 => Err(FdtabError::BadInputFd(">0".to_owned(), fd))?,
        1 | 2 => return Ok(libos!(stdout(buf))),
        _ => {}
    }

    FD_TABLE.with_file(fd, |file| -> FdtabResult<Size> {
        let file = file.ok_or(FdtabError::NoExistFd(fd))?;

        if !file.can_write() {
            Err(FdtabError::NoWritePerm(fd))?
        }

        Ok(match file.src {
            DataSource::FatFS(raw_fd) => libos!(fatfs_write(raw_fd, buf))?,
            DataSource::Net(sockfd) => libos!(send(sockfd, buf)).map(|_| buf.len())?,
        })
    })
}

#[no_mangle]
pub fn lseek(fd: Fd, pos: u32) -> FdtabResult<()> {
    if let 0..=2 = fd {
        Err(FdtabError::BadInputFd(">2".to_owned(), fd))?
    }

    FD_TABLE.with_file(fd, |file| -> FdtabResult<()> {
        let file = file.ok_or(FdtabError::NoExistFd(fd))?;

        match file.src {
            DataSource::FatFS(raw_fd) => libos!(fatfs_seek(raw_fd, pos))?,
            DataSource::Net(_) => Err(FdtabError::UndefinedOperation {
                op: "lseek".to_owned(),
                fd,
                fd_type: "Net".to_owned(),
            })?,
        };
        Ok(())
    })
}

#[no_mangle]
pub fn stat(fd: Fd) -> FdtabResult<Stat> {
    if let 0..=2 = fd {
        Err(FdtabError::BadInputFd(">2".to_owned(), fd))?
    }

    FD_TABLE.with_file(fd, |file| -> FdtabResult<Stat> {
        let file = file.ok_or(FdtabError::NoExistFd(fd))?;

        Ok(match file.src {
            DataSource::FatFS(raw_fd) => libos!(fatfs_stat(raw_fd))?,
            DataSource::Net(_) => Err(FdtabError::UndefinedOperation {
                op: "stat".to_owned(),
                fd,
                fd_type: "Net".to_owned(),
            })?,
        })
    })
}

#[no_mangle]
pub fn open(path: &str, flags: OpenFlags, mode: OpenMode) -> FdtabResult<Fd> {
    let raw_fd = libos!(fatfs_open(path, flags))?;
    let file = File {
        mode,
        src: DataSource::FatFS(raw_fd),
    };

    Ok(FD_TABLE.add_file(file))
}

#[no_mangle]
pub fn close(fd: Fd) -> FdtabResult<()> {
    if let 0..=2 = fd {
        Err(FdtabError::BadInputFd(">2".to_owned(), fd))?
    }

    let file = FD_TABLE.remove_file(fd).ok_or(FdtabError::NoExistFd(fd))?;

    match file.src {
        DataSource::FatFS(raw_fd) => libos!(fatfs_close(raw_fd))?,
        DataSource::Net(socket) => libos!(smol_close(socket))?,
    };
    Ok(())
}

#[no_mangle]
pub fn connect(addr: SocketAddrV4) -> FdtabResult<Fd> {
    let sockfd = libos!(smol_connect(addr))?;

    let file = File {
        mode: OpenMode::RDWR,
        src: DataSource::Net(sockfd),
    };

    Ok(FD_TABLE.add_file(file))
}

#[no_mangle]
pub fn bind(addr: SocketAddrV4) -> FdtabResult<SockFd> {
    let listened_sockfd = libos!(smol_bind(addr))?;

    let file = File {
        mode: OpenMode::RD,
        src: DataSource::Net(listened_sockfd),
    };

    Ok(FD_TABLE.add_file(file))
}

#[no_mangle]
pub fn accept(listened_sockfd: SockFd) -> FdtabResult<SockFd> {
    if let 0..=2 = listened_sockfd {
        Err(FdtabError::BadInputFd(">2".to_owned(), listened_sockfd))?
    }

    let listened_sockfd =
        FD_TABLE.with_file_mut(listened_sockfd, |file| -> FdtabResult<SockFd> {
            let old_sock = file.ok_or(FdtabError::NoExistFd(listened_sockfd))?;

            if let DataSource::Net(sockfd) = old_sock.src {
                // old file is still listened socket, with new socket handle.
                old_sock.src = DataSource::Net(libos!(smol_accept(sockfd))?);
                // println!("sockfd is {}", sockfd);
                Ok(sockfd)
            } else {
                Err(FdtabError::UndefinedOperation {
                    op: "accept".to_owned(),
                    fd: listened_sockfd,
                    fd_type: "Net".to_owned(),
                })?
            }
        })?;

    // new file will be connected socket, with old socket handle.
    let new_sock = File {
        mode: OpenMode::RDWR,
        src: DataSource::Net(listened_sockfd),
    };
    let connected_sockfd = FD_TABLE.add_file(new_sock);

    Ok(connected_sockfd)
}
