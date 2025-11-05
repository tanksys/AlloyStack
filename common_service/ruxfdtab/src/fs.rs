use alloc::sync::Arc;
use axerrno::LinuxResult;
use axio::PollState;
use as_hostcall::{
    fdtab::{FdtabError, FdtabResult},
    types::Fd,
};
use ruxfdtable::{FileLike, RuxStat};
use spin::Mutex;

use crate::ctypes;

pub struct File {
    pub(crate) inner: Mutex<ruxfs::fops::File>,
}

impl File {
    pub(crate) fn new(inner: ruxfs::fops::File) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }

    pub(crate) fn add_to_fd_table(self) -> FdtabResult<Fd> {
        super::fd_ops::add_file_like(Arc::new(self))
    }

    pub(crate) fn from_fd(fd: Fd) -> FdtabResult<Arc<Self>> {
        let f = super::fd_ops::get_file_like(fd)?;
        f.into_any()
            .downcast::<Self>()
            .map_err(|_| FdtabError::NoExistFd(fd))
    }
}

impl FileLike for File {
    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        Ok(self.inner.lock().read(buf)?)
    }

    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        Ok(self.inner.lock().write(buf)?)
    }

    fn flush(&self) -> LinuxResult {
        Ok(self.inner.lock().flush()?)
    }

    fn stat(&self) -> LinuxResult<RuxStat> {
        let metadata = self.inner.lock().get_attr()?;
        let ty = metadata.file_type() as u8;
        let perm = metadata.perm().bits() as u32;
        let st_mode = ((ty as u32) << 12) | perm;

        // Inode of files, for musl dynamic linker.
        // WARN: there will be collision for files with the same size.
        // TODO: implement real inode.
        let st_ino = metadata.size() + st_mode as u64;

        let res = RuxStat::from(ctypes::stat {
            st_ino,
            st_nlink: 1,
            st_mode,
            st_uid: 1000,
            st_gid: 1000,
            st_size: metadata.size() as _,
            st_blocks: metadata.blocks() as _,
            st_blksize: 512,
            ..Default::default()
        });

        Ok(res)
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
        })
    }

    fn set_nonblocking(&self, _nonblocking: bool) -> LinuxResult {
        Ok(())
    }
}
