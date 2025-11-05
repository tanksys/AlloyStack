use std::{
    ffi::c_void,
    mem::MaybeUninit,
    os::fd::{AsFd, BorrowedFd},
    u64,
};

use nix::sys::epoll::{Epoll, EpollCreateFlags, EpollEvent, EpollFlags};
use userfaultfd::{Uffd, UffdBuilder};

use as_hostcall::{
    mmap_file_backend::{MmapFileErr, MmapFileResult},
    types::Fd,
};
use as_std::libos::libos;

use crate::{
    acquire_regions_or_notify, acquire_register, do_page_fault, init_notify_pipe, read_notify_pipe,
    Page, RegisterdMemRegion, NOTIFY_PIPE, REGISTERD_REGIONS,
};

#[no_mangle]
pub fn file_page_fault_handler() {
    let notify_fd =
        unsafe { BorrowedFd::borrow_raw(NOTIFY_PIPE.read().unwrap().as_ref().unwrap().recevier) };

    let mut page: Box<MaybeUninit<Page>> = Box::new(MaybeUninit::uninit());

    loop {
        let epoll = Epoll::new(EpollCreateFlags::empty()).expect("create epoll failed");

        let regions = REGISTERD_REGIONS.lock().unwrap();
        if regions.is_empty() {
            break;
        }

        let uffd_events: Vec<_> = regions
            .iter()
            .map(|region| region.uffd.as_fd())
            .enumerate()
            .collect();
        let notify_event = [(u64::MAX as usize, notify_fd.as_fd())];

        for (idx, fd) in uffd_events.iter().chain(notify_event.iter()) {
            epoll
                .add(fd, EpollEvent::new(EpollFlags::EPOLLIN, *idx as u64))
                .expect("add event failed");
        }

        let mut ready_events = [EpollEvent::empty()];
        epoll
            .wait(&mut ready_events, -1)
            .expect("epoll wait failed");
        // let revents = pollfd.revents().unwrap();

        if !ready_events[0].events().contains(EpollFlags::EPOLLIN) {
            continue;
        }
        let event_idx = ready_events[0].data();
        if let Some(region) = regions.get(event_idx as usize) {
            do_page_fault(page.as_mut_ptr() as usize as *mut u8, region)
                .expect("should panic, do page fault failed.")
        } else {
            drop(regions);
            let pipe = NOTIFY_PIPE.read().unwrap();
            let pipe = pipe.as_ref().expect("pipe not exist?");
            pipe.consume()
        }
    }

    let mut notify_pipe = NOTIFY_PIPE.write().unwrap();
    *notify_pipe = None;
    // println!("page fault handler exit.");
}

#[no_mangle]
pub fn register_file_backend(mm_region: &mut [c_void], file_fd: Fd) -> MmapFileResult<()> {
    let _lock = acquire_register();
    // If have error: `OpenDevUserfaultfd(Os { code: 13, kind: PermissionDenied, message: "Permission denied" })`
    // ,use this command:
    //    `setfacl -m u:${USER}:rw /dev/userfaultfd`
    let uffd = || -> userfaultfd::Result<Uffd> {
        let fd = UffdBuilder::new()
            .close_on_exec(true)
            .non_blocking(true)
            .user_mode_only(true)
            .create()?;

        fd.register(mm_region.as_mut_ptr(), mm_region.len())?;
        Ok(fd)
    }()
    .map_err(|e| MmapFileErr::UffdError(e.to_string()))?;

    let mut regions = acquire_regions_or_notify()?;
    regions.push(RegisterdMemRegion {
        uffd,
        start_addr: mm_region.as_ptr() as usize,
        src_fd: file_fd,
    });

    if read_notify_pipe()?.is_none() {
        init_notify_pipe()?;

        libos!(spawn_fault_handler(
            as_std::init_context::isolation_ctx().isol_id
        ))?;
    }
    // println!("spawn_fault_handler successfully.");

    Ok(())
}

#[no_mangle]
pub fn unregister_file_backend(addr: usize) -> MmapFileResult<()> {
    let _lock = acquire_register()?;
    
    let pipe = read_notify_pipe()?;
    let pipe = pipe
        .as_ref()
        .ok_or(MmapFileErr::PipeStateErr("uninit".to_owned()))?;
    pipe.notify();

    let mut regions = acquire_regions_or_notify()?;

    for (idx, region) in (*regions).iter().enumerate() {
        if region.start_addr == addr {
            regions.remove(idx);
            break;
        }
    }

    Ok(())
}
