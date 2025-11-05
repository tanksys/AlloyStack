#![feature(seek_stream_len)]

use std::{
    fs,
    mem::ManuallyDrop,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use fscommon::BufStream;

use as_hostcall::{fatfs::FatfsError, types::Fd};
use as_std::libos::libos;

type FileSystem = fatfs::FileSystem<fscommon::BufStream<std::fs::File>>;
type File<'a> = fatfs::File<'a, fscommon::BufStream<std::fs::File>>;
type FatfsHandle = usize;

pub mod apis;
pub mod drop_fs;

fn get_fs_image_path() -> PathBuf {
    let image_path = match libos!(fs_image(as_std::init_context::isolation_ctx().isol_id)) {
        Some(s) => s,
        None => "fs_images/fatfs.img".to_owned(),
    };

    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
        .join(image_path)
}

#[derive(Default)]
struct FatfsFileList {
    table: Vec<Option<usize>>,
}

impl FatfsFileList {
    const fn new() -> Self {
        Self { table: Vec::new() }
    }

    fn get_file_raw_ptr(&self, fd: Fd) -> Option<usize> {
        if let Some(Some(file_addr)) = self.table.get(fd as usize) {
            // println!("get_file_mut: file addr=0x{:x}", file_addr);
            Some(*file_addr)
        } else {
            None
        }
    }

    fn _get_file(&self, fd: Fd) -> Option<&'static File<'static>> {
        self.get_file_raw_ptr(fd)
            .map(|ptr| unsafe { &*(ptr as *const File) })
    }

    fn get_file_mut(&mut self, fd: Fd) -> Option<&'static mut File<'static>> {
        self.get_file_raw_ptr(fd)
            .map(|ptr| unsafe { &mut *(ptr as *mut File) })
    }

    fn add_file(&mut self, file: &File) -> FatfsHandle {
        self.table.push(Some(file as *const _ as usize));
        self.table.len() - 1
    }

    fn remove_file(&mut self, handle: FatfsHandle) -> Result<(), FatfsError> {
        let file = self.table.get_mut(handle);
        if let Some(file) = file {
            match file.take() {
                Some(file_addr) => {
                    let _ = unsafe { Box::from_raw(file_addr as *mut File) };
                    Ok(())
                }
                None => Err(FatfsError::BadInputFd(handle as Fd)),
            }
        } else {
            Err(FatfsError::BadInputFd(handle as Fd))
        }
    }
}

static FTABLE: Mutex<FatfsFileList> = Mutex::new(FatfsFileList::new());
static FS_REF_ADDR: OnceLock<usize> = OnceLock::new();

fn has_fs_ref() -> bool {
    FS_REF_ADDR.get().is_some()
}

fn get_fs_ref() -> &'static FileSystem {
    let ref_addr = FS_REF_ADDR.get_or_init(|| {
        // I think this hack for getting reference to file system instance is
        // valid because `ManuallyDrop` can guarantee 'static lifetime.
        let mut file_system: ManuallyDrop<Box<_>> = ManuallyDrop::new(Box::new({
            let image = {
                let mut config = fs::File::options();
                let image_path = get_fs_image_path();
                BufStream::new(
                    config
                        .read(true)
                        .write(true)
                        .open(image_path.clone())
                        .unwrap_or_else(|e| panic!("open img {:?} failed, err: {}", image_path, e)),
                )
            };
            FileSystem::new(image, fatfs::FsOptions::new()).expect("fatfs::new() failed.")
        }));

        file_system.as_mut() as *mut _ as usize
    });
    unsafe { &*(*ref_addr as *const FileSystem) }
}
