#![feature(error_in_core)]

use std::{
    ffi::c_void,
    os::fd::RawFd,
    slice::from_raw_parts_mut,
    sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use nix::fcntl::{fcntl, FcntlArg, OFlag};
use userfaultfd::{Event, Uffd};

use as_hostcall::{
    mmap_file_backend::{MmapFileErr, MmapFileResult},
    types::Fd,
};
use as_std::libos::libos;

pub mod apis;

#[repr(C, align(4096))]
struct Page([u8; PAGE_SIZE]);

const PAGE_SIZE: usize = 0x1000;

struct NotifyPipe {
    recevier: RawFd,
    sender: RawFd,
}

impl NotifyPipe {
    fn create() -> MmapFileResult<Self> {
        let (recevier, sender) = || -> Result<(i32, i32), Box<dyn core::error::Error>> {
            let (recevier, sender) = nix::unistd::pipe()?;
            fcntl(sender, FcntlArg::F_SETFL(OFlag::O_NONBLOCK))?;

            Ok((recevier, sender))
        }()
        .map_err(|e| MmapFileErr::NixErr(e.to_string()))?;

        Ok(Self { recevier, sender })
    }

    fn consume(&self) {
        let mut buf = [0u8];
        unsafe { nix::libc::read(self.recevier, buf.as_mut_ptr() as usize as *mut c_void, 1) };
    }

    fn notify(&self) {
        let buf = [0u8];
        unsafe { nix::libc::write(self.sender, buf.as_ptr() as usize as *mut c_void, 1) };
    }
}

impl Drop for NotifyPipe {
    fn drop(&mut self) {
        unsafe {
            nix::libc::close(self.recevier);
            nix::libc::close(self.sender);
        }
    }
}

static REGISTER: Mutex<()> = Mutex::new(());
static NOTIFY_PIPE: RwLock<Option<NotifyPipe>> = RwLock::new(None);
static REGISTERD_REGIONS: Mutex<Vec<RegisterdMemRegion>> = Mutex::new(Vec::new());

#[derive(Debug)]
struct RegisterdMemRegion {
    uffd: Uffd,
    start_addr: usize,
    src_fd: Fd,
}

fn read_at_offset(fd: Fd, offset: u32, page: *mut u8) -> MmapFileResult<()> {
    // Copy the page pointed to by 'page' into the faulting region. Vary the contents that are
    // copied in, so that it is more obvious that each fault is handled separately.
    let page = unsafe { from_raw_parts_mut(page, PAGE_SIZE) };

    libos!(lseek(fd, offset))?;
    let _read_size = libos!(read(fd, page))?;
    // println!(
    //     "src_file aligned_offset={}, read {} bytes",
    //     offset, read_size
    // );
    Ok(())
}

fn do_page_fault(page: *mut u8, region: &RegisterdMemRegion) -> MmapFileResult<()> {
    let uffd = &region.uffd;
    let event = uffd
        .read_event()
        .map_err(|e| MmapFileErr::UffdError(e.to_string()))?
        .ok_or(MmapFileErr::Unknown("uffd_msg should ready".to_owned()))?;

    if let Event::Pagefault { addr, .. } = event {
        // println!(
        //     "UFFD_EVENT_PAGEFAULT event: {:?}, register_info: {:?}",
        //     event, region
        // );
        let offset = addr as usize - region.start_addr;
        let aligned_offset = offset & (!PAGE_SIZE + 1);
        read_at_offset(region.src_fd, aligned_offset as u32, page)?;

        let dst = (addr as usize & !(PAGE_SIZE - 1)) as *mut c_void;
        let _copy = unsafe { uffd.copy(page as usize as *mut c_void, dst, PAGE_SIZE, true) }
            .map_err(|e| MmapFileErr::UffdError(e.to_string()))?;
        // println!("(uffdio_copy.copy returned {})", copy);
    } else {
        Err(MmapFileErr::Unknown(
            "Unexpected event on userfaultfd".to_owned(),
        ))?
    }

    Ok(())
}

fn init_notify_pipe() -> MmapFileResult<()> {
    let mut notify_pipe = write_notify_pipe()?;

    if notify_pipe.is_some() {
        Err(MmapFileErr::PipeStateErr("multi init".to_owned()))?
    }
    let pipe = NotifyPipe::create()?;
    *notify_pipe = Some(pipe);

    Ok(())
}

fn acquire_register() -> MmapFileResult<MutexGuard<'static, ()>> {
    REGISTER
        .lock()
        .map_err(|e| MmapFileErr::AcquireLockErr("REGISTER".to_owned(), e.to_string()))
}

fn acquire_regions() -> MmapFileResult<MutexGuard<'static, Vec<RegisterdMemRegion>>> {
    REGISTERD_REGIONS
        .lock()
        .map_err(|e| MmapFileErr::AcquireLockErr("REGISTERD_REGIONS".to_owned(), e.to_string()))
}

fn acquire_regions_or_notify() -> MmapFileResult<MutexGuard<'static, Vec<RegisterdMemRegion>>> {
    match REGISTERD_REGIONS.try_lock() {
        Ok(regions) => Ok(regions),
        Err(_) => {
            let notify_pipe = read_notify_pipe()?;
            notify_pipe
                .as_ref()
                .ok_or(MmapFileErr::PipeStateErr("uninit".to_owned()))?
                .notify();

            acquire_regions()
        }
    }
}

fn read_notify_pipe() -> MmapFileResult<RwLockReadGuard<'static, Option<NotifyPipe>>> {
    NOTIFY_PIPE
        .read()
        .map_err(|e| MmapFileErr::AcquireLockErr("NOTIFY_PIPE.read".to_owned(), e.to_string()))
}

fn write_notify_pipe() -> MmapFileResult<RwLockWriteGuard<'static, Option<NotifyPipe>>> {
    NOTIFY_PIPE
        .write()
        .map_err(|e| MmapFileErr::AcquireLockErr("NOTIFY_PIPE.read".to_owned(), e.to_string()))
}
