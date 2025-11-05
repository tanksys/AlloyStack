use crate::libos::USER_HOST_CALL;

pub macro func_type {
    (metric) => (as_hostcall::types::MetricFunc),
    (fs_image) => (as_hostcall::types::FsImageFunc),
    (spawn_fault_handler) => (as_hostcall::types::SpawnFaultThreadFunc),
    (write) => (as_hostcall::fdtab::WriteFunc),
    (open) => (as_hostcall::fdtab::OpenFunc),
    (read) => (as_hostcall::fdtab::ReadFunc),
    (close) => (as_hostcall::fdtab::CloseFunc),
    (lseek) => (as_hostcall::fdtab::LseekFunc),
    (stat) => (as_hostcall::fdtab::StatFunc),
    (readdir) => (as_hostcall::fdtab::ReadDirFunc),
    (connect) => (as_hostcall::fdtab::ConnectFunc),
    (bind) => (as_hostcall::fdtab::BindFunc),
    (accept) => (as_hostcall::fdtab::AcceptFunc),
    (fatfs_open) => (as_hostcall::fatfs::FatfsOpenFunc),
    (fatfs_write) => (as_hostcall::fatfs::FatfsWriteFunc),
    (fatfs_read) => (as_hostcall::fatfs::FatfsReadFunc),
    (fatfs_close) => (as_hostcall::fatfs::FatfsCloseFunc),
    (fatfs_seek) => (as_hostcall::fatfs::FatfsSeekFunc),
    (fatfs_stat) => (as_hostcall::fatfs::FatfsStatFunc),
    (stdout) => (as_hostcall::types::HostStdioFunc),
    (addrinfo) => (as_hostcall::socket::SmoltcpAddrInfoFunc),
    (smol_connect) => (as_hostcall::socket::SmoltcpConnectFunc),
    (send) => (as_hostcall::socket::SmoltcpSendFunc),
    (recv) => (as_hostcall::socket::SmoltcpRecvFunc),
    (smol_bind) => (as_hostcall::socket::SmoltcpBindFunc),
    (smol_accept) => (as_hostcall::socket::SmoltcpAcceptFunc),
    (smol_close) => (as_hostcall::socket::SmoltcpCloseFunc),
    (buffer_alloc) => (as_hostcall::mm::BufferAllocFunc),
    (access_buffer) => (as_hostcall::mm::AccessBufferFunc),
    (buffer_dealloc) => (as_hostcall::mm::BufferDeallocFunc),
    (mmap) => (as_hostcall::mm::MemmapFunc),
    (munmap) => (as_hostcall::mm::MemunmapFunc),
    (mprotect) => (as_hostcall::mm::MprotectFunc),
    (register_file_backend) => (as_hostcall::mmap_file_backend::RegisterFileBackendFunc),
    (unregister_file_backend) => (as_hostcall::mmap_file_backend::UnregisterFileBackendFunc),
    (get_time) => (as_hostcall::types::GetTimeFunc),
    (nanosleep) => (as_hostcall::types::NanoSleepFunc),
    (sigaction) => (as_hostcall::signal::SigActionFunc),
}

pub macro hostcall_id {
    (metric) => (as_hostcall::CommonHostCall::Metric),
    (fs_image) => (as_hostcall::CommonHostCall::FsImage),
    (spawn_fault_handler) => (as_hostcall::CommonHostCall::SpawnFaultThread),
    (write) => (as_hostcall::CommonHostCall::Write),
    (open) => (as_hostcall::CommonHostCall::Open),
    (read) => (as_hostcall::CommonHostCall::Read),
    (close) => (as_hostcall::CommonHostCall::Close),
    (lseek) => (as_hostcall::CommonHostCall::Lseek),
    (stat) => (as_hostcall::CommonHostCall::Stat),
    (readdir) => (as_hostcall::CommonHostCall::ReadDir),
    (connect) => (as_hostcall::CommonHostCall::Connect),
    (bind) => (as_hostcall::CommonHostCall::Bind),
    (accept) => (as_hostcall::CommonHostCall::Accept),
    (fatfs_open) => (as_hostcall::CommonHostCall::FatfsOpen),
    (fatfs_write) => (as_hostcall::CommonHostCall::FatfsWrite),
    (fatfs_read) => (as_hostcall::CommonHostCall::FatfsRead),
    (fatfs_close) => (as_hostcall::CommonHostCall::FatfsClose),
    (fatfs_seek) => (as_hostcall::CommonHostCall::FatfsSeek),
    (fatfs_stat) => (as_hostcall::CommonHostCall::FatfsStat),
    (stdout) => (as_hostcall::CommonHostCall::Stdout),
    (addrinfo) => (as_hostcall::CommonHostCall::SmoltcpAddrInfo),
    (smol_connect) => (as_hostcall::CommonHostCall::SmoltcpConnect),
    (send) => (as_hostcall::CommonHostCall::SmoltcpSend),
    (recv) => (as_hostcall::CommonHostCall::SmoltcpRecv),
    (smol_bind) => (as_hostcall::CommonHostCall::SmoltcpBind),
    (smol_accept) => (as_hostcall::CommonHostCall::SmoltcpAccept),
    (smol_close) => (as_hostcall::CommonHostCall::SmoltcpClose),
    (buffer_alloc) => (as_hostcall::CommonHostCall::BufferAlloc),
    (access_buffer) => (as_hostcall::CommonHostCall::AccessBuffer),
    (buffer_dealloc) => (as_hostcall::CommonHostCall::BufferDealloc),
    (mmap) => (as_hostcall::CommonHostCall::Mmap),
    (munmap) => (as_hostcall::CommonHostCall::Munmap),
    (mprotect) => (as_hostcall::CommonHostCall::Mprotect),
    (register_file_backend) => (as_hostcall::CommonHostCall::RegisterFileBackend),
    (unregister_file_backend) => (as_hostcall::CommonHostCall::UnregisterFileBackend),
    (get_time) => (as_hostcall::CommonHostCall::GetTime),
    (nanosleep) => (as_hostcall::CommonHostCall::NanoSleep),
    (sigaction) => (as_hostcall::CommonHostCall::SigAction),
}

pub macro libos {
    ($name:ident($($arg_name:expr),*)) => {
        {
            fn binding() -> func_type!($name){
                let mut table = USER_HOST_CALL.lock();
                unsafe { core::mem::transmute(table.get_or_find(hostcall_id!($name))) }
            }
            let $name = binding();
            let res = $name($($arg_name),*);
            res
        }
    }
}

#[cfg(feature = "mpk")]
pub macro libos_with_switch_mpk {
    ($name:ident($($arg_name:expr),*)) => {
        {
            use core::arch::asm;
            use crate::mpk;
            let pkru = mpk::pkey_read();
            let is_privilege_level = (pkru >> 30 == 0);
            // grant access to libos. 00 00 11 ... ... 11 00
            let pkru = mpk::grant_libos_perm(pkru);
            unsafe{
                asm!(
                    // "mov eax, 0x0FFFFFFC",
                    "xor rcx, rcx",
                    "mov rdx, rcx",
                    "wrpkru",
                    in("rax") pkru,
                );
            }

            fn binding() -> func_type!($name){
                let mut table = USER_HOST_CALL.lock();
                unsafe { core::mem::transmute(table.get_or_find(hostcall_id!($name))) }
            }
            let $name = binding();
            let res = $name($($arg_name),*);

            if !is_privilege_level {
                let pkru = mpk::drop_libos_perm(pkru);
                // drop permission to libos. 11 00 11 ... ... 11 00
                unsafe{
                    asm!(
                        // "mov eax, 0xCFFFFFFC",
                        "xor rcx, rcx",
                        "mov rdx, rcx",
                        in("rax") pkru,
                    );
                }
            }

            res
        }
    }
}
