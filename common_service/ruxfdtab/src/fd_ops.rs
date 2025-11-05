use alloc::sync::Arc;
use as_hostcall::{
    fdtab::{FdtabError, FdtabResult},
    types::Fd,
};
use ruxfdtable::{FileLike, RuxStat, RuxTimeSpec, FD_TABLE};

use crate::{
    ctypes,
    stdio::{stdin, stdout},
};

lazy_static::lazy_static! {
    static ref MUST_EXEC: usize = {
        FD_TABLE.write().add_at(0, Arc::new(stdin()) as _).unwrap(); // stdin
        FD_TABLE.write().add_at(1, Arc::new(stdout()) as _).unwrap(); // stdout
        FD_TABLE.write().add_at(2, Arc::new(stdout()) as _).unwrap(); // stderr
        0
    };
}

impl From<ctypes::timespec> for RuxTimeSpec {
    fn from(ctimespec: ctypes::timespec) -> Self {
        RuxTimeSpec {
            tv_sec: ctimespec.tv_sec,
            tv_nsec: ctimespec.tv_nsec,
        }
    }
}

impl From<ctypes::stat> for RuxStat {
    fn from(cstat: ctypes::stat) -> Self {
        RuxStat {
            st_dev: cstat.st_dev,
            st_ino: cstat.st_ino,
            st_nlink: cstat.st_nlink,
            st_mode: cstat.st_mode,
            st_uid: cstat.st_uid,
            st_gid: cstat.st_gid,
            __pad0: cstat.__pad0,
            st_rdev: cstat.st_rdev,
            st_size: cstat.st_size,
            st_blksize: cstat.st_blksize,
            st_blocks: cstat.st_blocks,
            st_atime: RuxTimeSpec::from(cstat.st_atime),
            st_mtime: RuxTimeSpec::from(cstat.st_mtime),
            st_ctime: RuxTimeSpec::from(cstat.st_ctime),
            __unused: cstat.__unused,
        }
    }
}

pub fn get_file_like(fd: Fd) -> FdtabResult<Arc<dyn FileLike>> {
    let _exec = *MUST_EXEC;
    FD_TABLE
        .read()
        .get(fd as usize)
        .cloned()
        .ok_or(FdtabError::NoExistFd(fd))
}

pub fn add_file_like(f: Arc<dyn FileLike>) -> FdtabResult<Fd> {
    let _exec = *MUST_EXEC;
    let err_msg = FdtabError::RuxfsError("add_file_like failed".to_owned());
    Ok(FD_TABLE.write().add(f).ok_or(err_msg)? as u32)
}

pub fn close_file_like(fd: Fd) -> FdtabResult<()> {
    let _exec = *MUST_EXEC;
    let err_msg = FdtabError::RuxfsError("close_file_like failed".to_owned());
    let f = FD_TABLE.write().remove(fd as usize).ok_or(err_msg)?;
    drop(f);
    Ok(())
}
