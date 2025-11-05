pub type SigActionFunc = unsafe extern "C" fn(i32, *const SigAction, *const SigAction) -> i32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SigInfo {
    pub siginfo: SigInfoIntern,
    si_pad: [u8; 128],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SigAction {
    pub sa_handler: usize,
    pub sa_flags: usize,
    // pub sa_restorer: Option<unsafe extern "C" fn()>,
    pub sa_mask: Sigset,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Ucontext {
    pub uc_flags: u64,
    pub uc_link: *const Ucontext,
    pub uc_stack: StackT,
    pub uc_mcontext: Mcontext,
    pub uc_sigmask: Sigset,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct StackT {
    pub ss_sp: *mut u8,
    pub ss_flags: i32,
    pub ss_size: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Mcontext {
    pub gregs: [usize; 23],
    pub fpregs: *const u8,
    pub __reserved1: [usize; 8],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Sigset {
    pub sig: [usize; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SigInfoIntern {
    pub si_signo: i32,
    pub si_errno: i32,
    pub si_code: i32,
    pub sifields: Sifields,
}


#[repr(C)]
#[derive(Clone, Copy)]
pub union Sifields {
    pub kill: si_kill_t,

    pub timer: si_timer_t,

    pub rt: si_rt_t,

    pub sigchld: si_sigchld_t,
    pub sigpoll: si_sigpoll_t,

    pub sigsys: si_sigsys_t,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct si_rt_t {
    /// sender's pid
    pub pid: i32,
    /// sender's uid
    pub uid: u32,
    pub sigval: sigval_t,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct si_sigpoll_t {
    /// `POLL_IN`, `POLL_OUT`, `POLL_MSG`
    pub band: isize,
    pub fd: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union sigval_t {
    pub sival_int: i32,
    pub sival_ptr: usize,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct si_sigsys_t {
    pub call_addr: usize,
    pub syscall: i32,
    pub arch: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct si_timer_t {
    pub tid: i32,
    pub overrun: i32,
    pub sigval: sigval_t,
    sys_private: i32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct si_kill_t {
    pub pid: i32,
    pub uid: u32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct si_sigchld_t {
    pub pid: i32,
    pub uid: u32,
    pub status: i32,
    pub utime: isize,
    pub stime: isize,
}
