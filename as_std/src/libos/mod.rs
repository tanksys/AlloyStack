use core::mem::MaybeUninit;

use spin::Mutex;

use crate::init_context::isolation_ctx;
pub use as_hostcall::types::MetricEvent;
use as_hostcall::{
    types::{FindHostCallFunc, PanicHandlerFunc, Transmutor},
    CommonHostCall, HostCallID,
};
mod utils;

#[cfg(not(feature = "mpk"))]
pub use utils::libos;
#[cfg(feature = "mpk")]
pub use utils::libos_with_switch_mpk as libos;

pub static USER_HOST_CALL: Mutex<UserHostCall> = Mutex::new(UserHostCall::new());

#[derive(Default)]
pub struct UserHostCall {
    metric_addr: Option<usize>,
    fs_image_addr: Option<usize>,
    spawn_thread: Option<usize>,

    write_addr: Option<usize>,
    open_addr: Option<usize>,
    read_addr: Option<usize>,
    close_addr: Option<usize>,
    lseek_addr: Option<usize>,
    stat_addr: Option<usize>,
    readdir_addr: Option<usize>,
    connect_addr: Option<usize>,
    socket_addr: Option<usize>,
    bind_addr: Option<usize>,
    accept_addr: Option<usize>,

    stdout_addr: Option<usize>,

    fatfs_open_addr: Option<usize>,
    fatfs_write_addr: Option<usize>,
    fatfs_read_addr: Option<usize>,
    fatfs_close_addr: Option<usize>,
    fatfs_seek_addr: Option<usize>,
    fatfs_stat_addr: Option<usize>,

    smoltcp_addrinfo_addr: Option<usize>,
    smoltcp_connect_addr: Option<usize>,
    smoltcp_send_addr: Option<usize>,
    smoltcp_recv_addr: Option<usize>,
    smoltcp_bind_addr: Option<usize>,
    smoltcp_accept_addr: Option<usize>,
    smoltcp_close_addr: Option<usize>,

    alloc_buffer_addr: Option<usize>,
    access_buffer_addr: Option<usize>,
    dealloc_buffer_addr: Option<usize>,
    mmap_addr: Option<usize>,
    munmap_addr: Option<usize>,
    mprotect_addr: Option<usize>,

    pf_handler_addr: Option<usize>,
    register_file_backend_addr: Option<usize>,
    unregister_file_backend_addr: Option<usize>,

    get_time_addr: Option<usize>,
    nanosleep_addr: Option<usize>,

    sigaction_addr: Option<usize>,
}

impl UserHostCall {
    const fn new() -> Self {
        let v: MaybeUninit<Self> = MaybeUninit::zeroed();
        unsafe { v.assume_init() }
    }
}

impl UserHostCall {
    pub fn get_or_find(&mut self, chc_id: CommonHostCall) -> usize {
        let entry_addr = match chc_id {
            CommonHostCall::Metric => &mut self.metric_addr,
            CommonHostCall::FsImage => &mut self.fs_image_addr,
            CommonHostCall::SpawnFaultThread => &mut self.spawn_thread,

            CommonHostCall::Write => &mut self.write_addr,
            CommonHostCall::Open => &mut self.open_addr,
            CommonHostCall::Read => &mut self.read_addr,
            CommonHostCall::Close => &mut self.close_addr,
            CommonHostCall::Lseek => &mut self.lseek_addr,
            CommonHostCall::Stat => &mut self.stat_addr,
            CommonHostCall::ReadDir => &mut self.readdir_addr,
            CommonHostCall::Connect => &mut self.connect_addr,
            CommonHostCall::Socket => &mut self.socket_addr,
            CommonHostCall::Bind => &mut self.bind_addr,
            CommonHostCall::Accept => &mut self.accept_addr,

            CommonHostCall::Stdout => &mut self.stdout_addr,

            CommonHostCall::FatfsOpen => &mut self.fatfs_open_addr,
            CommonHostCall::FatfsWrite => &mut self.fatfs_write_addr,
            CommonHostCall::FatfsRead => &mut self.fatfs_read_addr,
            CommonHostCall::FatfsClose => &mut self.fatfs_close_addr,
            CommonHostCall::FatfsSeek => &mut self.fatfs_seek_addr,
            CommonHostCall::FatfsStat => &mut self.fatfs_stat_addr,

            CommonHostCall::SmoltcpAddrInfo => &mut self.smoltcp_addrinfo_addr,
            CommonHostCall::SmoltcpConnect => &mut self.smoltcp_connect_addr,
            CommonHostCall::SmoltcpSend => &mut self.smoltcp_send_addr,
            CommonHostCall::SmoltcpRecv => &mut self.smoltcp_recv_addr,
            CommonHostCall::SmoltcpBind => &mut self.smoltcp_bind_addr,
            CommonHostCall::SmoltcpAccept => &mut self.smoltcp_accept_addr,
            CommonHostCall::SmoltcpClose => &mut self.smoltcp_close_addr,

            CommonHostCall::BufferAlloc => &mut self.alloc_buffer_addr,
            CommonHostCall::AccessBuffer => &mut self.access_buffer_addr,
            CommonHostCall::BufferDealloc => &mut self.dealloc_buffer_addr,
            CommonHostCall::Mmap => &mut self.mmap_addr,
            CommonHostCall::Munmap => &mut self.munmap_addr,
            CommonHostCall::Mprotect => &mut self.mprotect_addr,

            CommonHostCall::RegisterFileBackend => &mut self.register_file_backend_addr,
            CommonHostCall::UnregisterFileBackend => &mut self.unregister_file_backend_addr,
            CommonHostCall::FilePageFaultHandler => &mut self.pf_handler_addr,

            CommonHostCall::GetTime => &mut self.get_time_addr,
            CommonHostCall::NanoSleep => &mut self.nanosleep_addr,

            CommonHostCall::SigAction => &mut self.sigaction_addr,
        };

        if entry_addr.is_none() {
            let find_host_call = UserHostCall::find_host_call();
            let addr =
                unsafe { find_host_call(isolation_ctx().isol_id, HostCallID::Common(chc_id)) };
            *entry_addr = Some(addr);
            addr
        } else {
            entry_addr.unwrap()
        }
    }
}

impl Transmutor for UserHostCall {
    fn find_host_call() -> FindHostCallFunc {
        unsafe { core::mem::transmute(isolation_ctx().find_handler) }
    }

    fn host_panic_handler() -> PanicHandlerFunc {
        unsafe { core::mem::transmute(isolation_ctx().panic_handler) }
    }
}

pub fn metric(event: MetricEvent) {
    if !matches!(event, MetricEvent::Mem) {
        panic!("unallowed event")
    };

    if libos!(metric(isolation_ctx().isol_id, event)).is_err() {}
}
