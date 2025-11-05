use core::{mem::ManuallyDrop, slice};

use as_hostcall::{
    fdtab::FdtabError,
    mm::{MMError, ProtFlags},
};
use thiserror_no_std::Error;

use crate::{fs::File, libos::libos};

const PAGE_SIZE: usize = 0x1000;

pub struct Mmap {
    ptr: usize,
    length: usize,
}

#[derive(Debug, Error)]
pub enum MmapError {
    #[error(transparent)]
    FileErr(#[from] FdtabError),
    #[error(transparent)]
    MMErr(#[from] MMError),
}

impl Mmap {
    pub fn mmap_file(file: File) -> Result<Self, MmapError> {
        let file = ManuallyDrop::new(file);
        let length = file.metadata()?.st_size;
        let aligned_length = (length + PAGE_SIZE - 1) & (!PAGE_SIZE + 1);
        let ptr = libos!(mmap(0, aligned_length, ProtFlags::READ, file.as_raw_fd()))?;

        Ok(Mmap { ptr, length })
    }
}

impl AsRef<[u8]> for Mmap {
    fn as_ref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr as *const u8, self.length) }
    }
}

impl Drop for Mmap {
    fn drop(&mut self) {
        let region = unsafe { slice::from_raw_parts_mut(self.ptr as *mut u8, self.length) };
        libos!(munmap(region, true)).expect("munmap failed")
    }
}
