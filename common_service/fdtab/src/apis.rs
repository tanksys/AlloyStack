use core::net::SocketAddrV4;

use alloc::{borrow::ToOwned, string::String, vec, vec::Vec};
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
            DataSource::Directory { .. } => Err(FdtabError::UndefinedOperation {
                op: "read".to_owned(),
                fd,
                fd_type: "Directory".to_owned(),
            })?,
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
            DataSource::Directory { .. } => Err(FdtabError::UndefinedOperation {
                op: "write".to_owned(),
                fd,
                fd_type: "Directory".to_owned(),
            })?,
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
            DataSource::Directory { .. } => Err(FdtabError::UndefinedOperation {
                op: "lseek".to_owned(),
                fd,
                fd_type: "Directory".to_owned(),
            })?,
        };
        Ok(())
    })
}

#[no_mangle]
pub fn lseek64(fd: Fd, offset: i64, whence: i32) -> FdtabResult<i64> {
    if let 0..=2 = fd {
        Err(FdtabError::BadInputFd(">2".to_owned(), fd))?
    }

    FD_TABLE.with_file(fd, |file| -> FdtabResult<i64> {
        let file = file.ok_or(FdtabError::NoExistFd(fd))?;

        match file.src {
            DataSource::FatFS(raw_fd) => Ok(libos!(fatfs_seek64(raw_fd, offset, whence))?),
            DataSource::Net(_) => Err(FdtabError::UndefinedOperation {
                op: "lseek64".to_owned(),
                fd,
                fd_type: "Net".to_owned(),
            }),
            DataSource::Directory { .. } => Err(FdtabError::UndefinedOperation {
                op: "lseek64".to_owned(),
                fd,
                fd_type: "Directory".to_owned(),
            }),
        }
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
            DataSource::Directory { .. } => Stat {
                st_dev: 0,
                st_ino: 0,
                st_nlink: 1,
                st_mode: 0o040755,
                st_uid: 0,
                st_gid: 0,
                __pad0: 0,
                st_rdev: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_atime: as_hostcall::types::TimeSpec {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtime: as_hostcall::types::TimeSpec {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctime: as_hostcall::types::TimeSpec {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                __unused: [0, 0, 0],
            },
        })
    })
}

#[no_mangle]
pub fn path_stat(path: &str) -> FdtabResult<Stat> {
    Ok(libos!(fatfs_path_stat(path))?)
}

#[no_mangle]
pub fn read_dir(path: &str) -> FdtabResult<alloc::vec::Vec<as_hostcall::types::DirEntry>> {
    let required = libos!(fatfs_readdir(path, &mut []))?;
    let mut packed = vec![0; required];
    let written = libos!(fatfs_readdir(path, &mut packed))?;
    packed.truncate(written);

    let mut entries = Vec::new();
    let mut cursor = 0;
    while cursor < packed.len() {
        if packed.len() - cursor < 3 {
            return Err(FdtabError::RuxfsError("invalid directory record".to_owned()));
        }
        let entry_type = packed[cursor] as u32;
        let name_len = u16::from_ne_bytes([packed[cursor + 1], packed[cursor + 2]]) as usize;
        cursor += 3;
        if name_len > packed.len() - cursor {
            return Err(FdtabError::RuxfsError("invalid directory name".to_owned()));
        }
        let entry_name = core::str::from_utf8(&packed[cursor..cursor + name_len])
            .map_err(|_| FdtabError::RuxfsError("non-UTF-8 directory name".to_owned()))?;
        entries.push(as_hostcall::types::DirEntry {
            dir_path: String::from(path),
            entry_name: String::from(entry_name),
            entry_type,
        });
        cursor += name_len;
    }
    Ok(entries)
}

#[no_mangle]
pub fn open_dir(path: &str) -> FdtabResult<Fd> {
    const HEADER_LEN: usize = 19;

    let portable = read_dir(path)?;
    let required = portable
        .iter()
        .map(|entry| (HEADER_LEN + entry.entry_name.len() + 1 + 7) & !7)
        .sum();
    let mut entries = vec![0; required];
    let mut cursor = 0;
    for (index, entry) in portable.iter().enumerate() {
        let name = entry.entry_name.as_bytes();
        let record_len = (HEADER_LEN + name.len() + 1 + 7) & !7;
        let record = &mut entries[cursor..cursor + record_len];
        record[0..8].copy_from_slice(&((index + 1) as u64).to_ne_bytes());
        record[8..16].copy_from_slice(&((index + 1) as i64).to_ne_bytes());
        record[16..18].copy_from_slice(&(record_len as u16).to_ne_bytes());
        record[18] = entry.entry_type as u8;
        record[HEADER_LEN..HEADER_LEN + name.len()].copy_from_slice(name);
        cursor += record_len;
    }
    Ok(FD_TABLE.add_file(File {
        mode: OpenMode::RD,
        src: DataSource::Directory { entries, cursor: 0 },
    }))
}

#[no_mangle]
pub fn getdents(fd: Fd, buffer: &mut [u8]) -> FdtabResult<Size> {
    FD_TABLE.with_file_mut(fd, |file| -> FdtabResult<Size> {
        let file = file.ok_or(FdtabError::NoExistFd(fd))?;
        let DataSource::Directory { entries, cursor } = &mut file.src else {
            return Err(FdtabError::UndefinedOperation {
                op: "getdents".to_owned(),
                fd,
                fd_type: "non-directory".to_owned(),
            });
        };

        let mut written = 0usize;
        while *cursor < entries.len() {
            if entries.len() - *cursor < 18 {
                return Err(FdtabError::RuxfsError("invalid dirent record".to_owned()));
            }
            let record_len =
                u16::from_ne_bytes([entries[*cursor + 16], entries[*cursor + 17]]) as usize;
            if record_len == 0 || record_len > entries.len() - *cursor {
                return Err(FdtabError::RuxfsError("invalid dirent length".to_owned()));
            }
            if record_len > buffer.len() - written {
                break;
            }
            buffer[written..written + record_len]
                .copy_from_slice(&entries[*cursor..*cursor + record_len]);
            *cursor += record_len;
            written += record_len;
        }
        Ok(written)
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
        DataSource::Directory { .. } => {}
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
