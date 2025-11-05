#![no_std]
#![feature(ip_in_core)]

extern crate alloc;

use alloc::vec::Vec;

use spin::Mutex;

use as_hostcall::types::{Fd, OpenMode, SockFd};
use as_std::{self, println};

pub mod apis;

#[derive(Clone)]
enum DataSource {
    FatFS(Fd),
    Net(SockFd),
}

#[derive(Clone)]
struct File {
    mode: OpenMode,
    src: DataSource,
}

impl File {
    fn can_read(&self) -> bool {
        // OpenMode::RDWR => 011;
        // OpenMode::RD   => 001;
        self.mode.contains(OpenMode::RD)
    }

    fn can_write(&self) -> bool {
        self.mode.contains(OpenMode::WR)
    }
}

static FD_TABLE: FdTable = FdTable::new();

struct FdTable {
    inner: Mutex<Vec<Option<File>>>,
}

impl FdTable {
    const fn new() -> Self {
        FdTable {
            inner: Mutex::new(Vec::new()),
        }
    }

    fn with_file<F, T>(&self, fd: Fd, f: F) -> T
    where
        F: FnOnce(Option<&File>) -> T,
    {
        let fdtab = self.inner.lock();
        // println!("get_file: fd={}, fd_table.len()={}", fd, fdtab.len());
        let file = if let Some(Some(file)) = fdtab.get(fd as usize - 3) {
            Some(file)
        } else {
            println!("file fd={} not exist?", fd);
            None
        };

        f(file)
    }

    fn with_file_mut<F, T>(&self, fd: Fd, f: F) -> T
    where
        F: FnOnce(Option<&mut File>) -> T,
    {
        // println!("get_file: fd={}, fd_table.len()={}", fd, fdtab.len());
        let mut fdtab = self.inner.lock();
        let file = if let Some(Some(file)) = fdtab.get_mut(fd as usize - 3) {
            Some(file)
        } else {
            println!("file fd={} not exist?", fd);
            None
        };

        f(file)
    }

    fn add_file(&self, file: File) -> Fd {
        let mut fdtab = self.inner.lock();
        for (idx, item) in fdtab.iter_mut().enumerate() {
            if item.is_some() {
                continue;
            }

            *item = Some(file);
            return (idx + 3) as Fd;
        }

        fdtab.push(Some(file));
        (fdtab.len() + 2) as Fd
    }

    fn remove_file(&self, fd: Fd) -> Option<File> {
        let mut fdtab = self.inner.lock();
        if fd as usize - 3 >= fdtab.len() {
            return None;
        }

        let old = fdtab[fd as usize - 3].clone();
        fdtab[fd as usize - 3] = None;

        old
    }
}
