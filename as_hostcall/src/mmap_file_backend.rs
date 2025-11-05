use core::ffi::c_void;

use alloc::string::String;
use thiserror_no_std::Error;

use crate::{fdtab::FdtabError, types::Fd};

pub type RegisterFileBackendFunc = fn(&mut [c_void], Fd) -> MmapFileResult<()>;
pub type UnregisterFileBackendFunc = fn(usize) -> MmapFileResult<()>;
pub type FilePageFaultHandlerFunc = fn();

pub type MmapFileResult<T> = Result<T, MmapFileErr>;

#[derive(Debug, Error)]
pub enum MmapFileErr {
    #[error("acquire name={0}, reason={1}")]
    AcquireLockErr(String, String),
    #[error("pipe error: {0}")]
    PipeStateErr(String),
    #[error("nix error: {0}")]
    NixErr(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
    #[error(transparent)]
    FileError(#[from] FdtabError),
    #[error(transparent)]
    SpawnHandlerError(#[from] String),
    #[error("userfaultfd error: {0}")]
    UffdError(String),
}
