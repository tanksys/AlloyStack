use std::{
    io::{Read, Seek, Write},
    mem::ManuallyDrop,
};

use as_hostcall::{
    fatfs::{FatfsError, FatfsResult},
    types::{Fd, OpenFlags, Size, Stat, TimeSpec},
};

use crate::{get_fs_ref, FTABLE};

#[no_mangle]
pub fn fatfs_read(fd: Fd, buf: &mut [u8]) -> FatfsResult<Size> {
    let mut table = FTABLE
        .lock()
        .map_err(|e| FatfsError::AcquireLockErr(e.to_string()))?;

    let file = table.get_file_mut(fd).ok_or(FatfsError::BadInputFd(fd))?;

    let mut read_size = 0;
    let mut buf = buf;
    while !buf.is_empty() {
        match file.read(buf) {
            Ok(0) => break,
            Ok(size) => {
                read_size += size;
                buf = &mut buf[size..]
            }
            Err(e) => Err(FatfsError::HostIOErr(e.to_string()))?,
        }
    }

    Ok(read_size)
}

#[no_mangle]
pub fn fatfs_seek(fd: Fd, pos: u32) -> FatfsResult<()> {
    // println!("fatfs: try seek to {}", pos);
    let mut table = FTABLE
        .lock()
        .map_err(|e| FatfsError::AcquireLockErr(e.to_string()))?;

    let f = table.get_file_mut(fd).ok_or(FatfsError::BadInputFd(fd))?;
    f.seek(std::io::SeekFrom::Start(pos as u64))
        .map_err(|e| FatfsError::HostIOErr(e.to_string()))?;

    Ok(())
}

#[no_mangle]
pub fn fatfs_stat(fd: Fd) -> FatfsResult<Stat> {
    let mut table = FTABLE
        .lock()
        .map_err(|e| FatfsError::AcquireLockErr(e.to_string()))?;

    let f = table.get_file_mut(fd).ok_or(FatfsError::BadInputFd(fd))?;

    let st_size = f
        .stream_len()
        .map_err(|e| FatfsError::HostIOErr(e.to_string()))? as Size;
    Ok(Stat {
        st_dev: 0,
        st_ino: 0,
        st_nlink: 0,
        st_mode: 0,
        st_uid: 0,
        st_gid: 0,
        __pad0: 0,
        st_rdev: 0,
        st_size: st_size,
        st_blksize: 0,
        st_blocks: 0,
        st_atime: TimeSpec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_mtime: TimeSpec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_ctime: TimeSpec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        __unused: [0, 0, 0],
    })
}

#[no_mangle]
pub fn fatfs_open(p: &str, flags: OpenFlags) -> FatfsResult<Fd> {
    let mut table = FTABLE
        .lock()
        .map_err(|e| FatfsError::AcquireLockErr(e.to_string()))?;

    let root_dir = get_fs_ref().root_dir();

    let file = if flags.contains(OpenFlags::O_CREAT) {
        root_dir.create_file(p)
    } else {
        root_dir.open_file(p)
    }
    .map_err(|_e| FatfsError::Unknown)?;

    let fd = {
        let file = ManuallyDrop::new(Box::new(file));
        table.add_file(file.as_ref())
    };

    Ok(fd as u32)
}

#[no_mangle]
pub fn fatfs_write(fd: Fd, buf: &[u8]) -> FatfsResult<Size> {
    let mut table = FTABLE
        .lock()
        .map_err(|e| FatfsError::AcquireLockErr(e.to_string()))?;

    let file = table.get_file_mut(fd).ok_or(FatfsError::BadInputFd(fd))?;

    let _ = || -> Result<(), std::io::Error> {
        file.write_all(buf)?;
        file.flush()
    }()
    .map_err(|e| FatfsError::HostIOErr(e.to_string()));

    Ok(buf.len())
}

#[no_mangle]
pub fn fatfs_close(fd: Fd) -> FatfsResult<()> {
    let mut table = FTABLE
        .lock()
        .map_err(|e| FatfsError::AcquireLockErr(e.to_string()))?;

    table.remove_file(fd as usize)
}

#[test]
fn fatfs_open_test() {
    let mut table = FTABLE.lock().expect("require lock failed.");
    let fd = fatfs_open("new_file.txt", OpenFlags::O_CREAT).expect("open file failed") as usize;
    let f = table.get_file_mut(fd as u32).unwrap();

    let mut buf = String::new();
    f.read_to_string(&mut buf).expect("read failed");
    // assert!(!buf.is_empty());
}

#[test]
fn fatfs_write_test() {
    let root_dir = get_fs_ref().root_dir();
    let mut file = root_dir
        .create_file("hello.txt")
        .expect("create file failed");

    assert!(file.write(b"Hello World!").expect("write failed") > 0);
}
