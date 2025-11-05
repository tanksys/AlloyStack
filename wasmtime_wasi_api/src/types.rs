#![allow(warnings)]

#[repr(C)]
pub(crate) struct WasiCiovec {
    pub(crate) buf: u32,
    pub(crate) buf_len: u32,
}

#[repr(C)]
pub(crate) struct WasiDirent {
    pub(crate) d_next: u64,
    pub(crate) d_ino: u64,
    pub(crate) d_namelen: u32,
    pub(crate) d_type: u8
}

#[repr(C)]
pub(crate) struct WasiFdstat {
    pub(crate) fs_filetype: u8,
    pub(crate) fs_flags: u16,
    pub(crate) fs_rights_base: u64,
    pub(crate) fs_rights_inheriting: u64,
}

#[repr(C)]
pub(crate) struct WasiFilestat {
    pub(crate) dev: u64,
    pub(crate) ino: u64,
    pub(crate) filetype: u8,
    pub(crate) nlink: u64,
    pub(crate) size: u64,
    pub(crate) atim: u64,
    pub(crate) mtim: u64,
    pub(crate) ctim: u64,
}

#[repr(C)]
pub(crate)  struct WasiPrestatDir {
    pub(crate) dirname_len: u32,
}

#[repr(C)]
pub(crate) struct WasiPrestatUt {
    pub(crate) dir: WasiPrestatDir,
}

#[repr(C)]
pub(crate) struct WasiPrestatT {
    pub(crate) tag: u8,
    pub(crate) u: WasiPrestatUt,
}

pub(crate) enum Errno {
    #[doc = " No error occurred. System call completed successfully."]
    Success,
    #[doc = " Argument list too long."]
    Toobig,
    #[doc = " Permission denied."]
    Access,
    #[doc = " Address in use."]
    Addrinuse,
    #[doc = " Address not available."]
    Addrnotavail,
    #[doc = " Address family not supported."]
    Afnosupport,
    #[doc = " Resource unavailable, or operation would block."]
    Again,
    #[doc = " Connection already in progress."]
    Already,
    #[doc = " Bad file descriptor."]
    Badf,
    #[doc = " Bad message."]
    Badmsg,
    #[doc = " Device or resource busy."]
    Busy,
    #[doc = " Operation canceled."]
    Canceled,
    #[doc = " No child processes."]
    Child,
    #[doc = " Connection aborted."]
    Connaborted,
    #[doc = " Connection refused."]
    Connrefused,
    #[doc = " Connection reset."]
    Connreset,
    #[doc = " Resource deadlock would occur."]
    Deadlk,
    #[doc = " Destination address required."]
    Destaddrreq,
    #[doc = " Mathematics argument out of domain of function."]
    Dom,
    #[doc = " Reserved."]
    Dquot,
    #[doc = " File exists."]
    Exist,
    #[doc = " Bad address."]
    Fault,
    #[doc = " File too large."]
    Fbig,
    #[doc = " Host is unreachable."]
    Hostunreach,
    #[doc = " Identifier removed."]
    Idrm,
    #[doc = " Illegal byte sequence."]
    Ilseq,
    #[doc = " Operation in progress."]
    Inprogress,
    #[doc = " Interrupted function."]
    Intr,
    #[doc = " Invalid argument."]
    Inval,
    #[doc = " I/O error."]
    Io,
    #[doc = " Socket is connected."]
    Isconn,
    #[doc = " Is a directory."]
    Isdir,
    #[doc = " Too many levels of symbolic links."]
    Loop,
    #[doc = " File descriptor value too large."]
    Mfile,
    #[doc = " Too many links."]
    Mlink,
    #[doc = " Message too large."]
    Msgsize,
    #[doc = " Reserved."]
    Multihop,
    #[doc = " Filename too long."]
    Nametoolong,
    #[doc = " Network is down."]
    Netdown,
    #[doc = " Connection aborted by network."]
    Netreset,
    #[doc = " Network unreachable."]
    Netunreach,
    #[doc = " Too many files open in system."]
    Nfile,
    #[doc = " No buffer space available."]
    Nobufs,
    #[doc = " No such device."]
    Nodev,
    #[doc = " No such file or directory."]
    Noent,
    #[doc = " Executable file format error."]
    Noexec,
    #[doc = " No locks available."]
    Nolck,
    #[doc = " Reserved."]
    Nolink,
    #[doc = " Not enough space."]
    Nomem,
    #[doc = " No message of the desired type."]
    Nomsg,
    #[doc = " Protocol not available."]
    Noprotoopt,
    #[doc = " No space left on device."]
    Nospc,
    #[doc = " Function not supported."]
    Nosys,
    #[doc = " The socket is not connected."]
    Notconn,
    #[doc = " Not a directory or a symbolic link to a directory."]
    Notdir,
    #[doc = " Directory not empty."]
    Notempty,
    #[doc = " State not recoverable."]
    Notrecoverable,
    #[doc = " Not a socket."]
    Notsock,
    #[doc = " Not supported, or operation not supported on socket."]
    Notsup,
    #[doc = " Inappropriate I/O control operation."]
    Notty,
    #[doc = " No such device or address."]
    Nxio,
    #[doc = " Value too large to be stored in data type."]
    Overflow,
    #[doc = " Previous owner died."]
    Ownerdead,
    #[doc = " Operation not permitted."]
    Perm,
    #[doc = " Broken pipe."]
    Pipe,
    #[doc = " Protocol error."]
    Proto,
    #[doc = " Protocol not supported."]
    Protonosupport,
    #[doc = " Protocol wrong type for socket."]
    Prototype,
    #[doc = " Result too large."]
    Range,
    #[doc = " Read-only file system."]
    Rofs,
    #[doc = " Invalid seek."]
    Spipe,
    #[doc = " No such process."]
    Srch,
    #[doc = " Reserved."]
    Stale,
    #[doc = " Connection timed out."]
    Timedout,
    #[doc = " Text file busy."]
    Txtbsy,
    #[doc = " Cross-device link."]
    Xdev,
    #[doc = " Extension: Capabilities insufficient."]
    Notcapable,
    #[doc = " Cannot send after socket shutdown."]
    Shutdown,
    #[doc = " Memory access violation."]
    Memviolation,
    #[doc = " An unknown error has occured"]
    Unknown,
}

impl Errno {
    pub fn name(&self) -> &'static str {
        match self {
            Errno::Success => "success",
            Errno::Toobig => "toobig",
            Errno::Access => "access",
            Errno::Addrinuse => "addrinuse",
            Errno::Addrnotavail => "addrnotavail",
            Errno::Afnosupport => "afnosupport",
            Errno::Again => "again",
            Errno::Already => "already",
            Errno::Badf => "badf",
            Errno::Badmsg => "badmsg",
            Errno::Busy => "busy",
            Errno::Canceled => "canceled",
            Errno::Child => "child",
            Errno::Connaborted => "connaborted",
            Errno::Connrefused => "connrefused",
            Errno::Connreset => "connreset",
            Errno::Deadlk => "deadlk",
            Errno::Destaddrreq => "destaddrreq",
            Errno::Dom => "dom",
            Errno::Dquot => "dquot",
            Errno::Exist => "exist",
            Errno::Fault => "fault",
            Errno::Fbig => "fbig",
            Errno::Hostunreach => "hostunreach",
            Errno::Idrm => "idrm",
            Errno::Ilseq => "ilseq",
            Errno::Inprogress => "inprogress",
            Errno::Intr => "intr",
            Errno::Inval => "inval",
            Errno::Io => "io",
            Errno::Isconn => "isconn",
            Errno::Isdir => "isdir",
            Errno::Loop => "loop",
            Errno::Mfile => "mfile",
            Errno::Mlink => "mlink",
            Errno::Msgsize => "msgsize",
            Errno::Multihop => "multihop",
            Errno::Nametoolong => "nametoolong",
            Errno::Netdown => "netdown",
            Errno::Netreset => "netreset",
            Errno::Netunreach => "netunreach",
            Errno::Nfile => "nfile",
            Errno::Nobufs => "nobufs",
            Errno::Nodev => "nodev",
            Errno::Noent => "noent",
            Errno::Noexec => "noexec",
            Errno::Nolck => "nolck",
            Errno::Nolink => "nolink",
            Errno::Nomem => "nomem",
            Errno::Nomsg => "nomsg",
            Errno::Noprotoopt => "noprotoopt",
            Errno::Nospc => "nospc",
            Errno::Nosys => "nosys",
            Errno::Notconn => "notconn",
            Errno::Notdir => "notdir",
            Errno::Notempty => "notempty",
            Errno::Notrecoverable => "notrecoverable",
            Errno::Notsock => "notsock",
            Errno::Notsup => "notsup",
            Errno::Notty => "notty",
            Errno::Nxio => "nxio",
            Errno::Overflow => "overflow",
            Errno::Ownerdead => "ownerdead",
            Errno::Perm => "perm",
            Errno::Pipe => "pipe",
            Errno::Proto => "proto",
            Errno::Protonosupport => "protonosupport",
            Errno::Prototype => "prototype",
            Errno::Range => "range",
            Errno::Rofs => "rofs",
            Errno::Spipe => "spipe",
            Errno::Srch => "srch",
            Errno::Stale => "stale",
            Errno::Timedout => "timedout",
            Errno::Txtbsy => "txtbsy",
            Errno::Xdev => "xdev",
            Errno::Notcapable => "notcapable",
            Errno::Shutdown => "shutdown",
            Errno::Memviolation => "memviolation",
            Errno::Unknown => "unknown",
        }
    }
    pub fn message(&self) -> &'static str {
        match self {
            Errno::Success => "No error occurred. System call completed successfully.",
            Errno::Toobig => "Argument list too long.",
            Errno::Access => "Permission denied.",
            Errno::Addrinuse => "Address in use.",
            Errno::Addrnotavail => "Address not available.",
            Errno::Afnosupport => "Address family not supported.",
            Errno::Again => "Resource unavailable, or operation would block.",
            Errno::Already => "Connection already in progress.",
            Errno::Badf => "Bad file descriptor.",
            Errno::Badmsg => "Bad message.",
            Errno::Busy => "Device or resource busy.",
            Errno::Canceled => "Operation canceled.",
            Errno::Child => "No child processes.",
            Errno::Connaborted => "Connection aborted.",
            Errno::Connrefused => "Connection refused.",
            Errno::Connreset => "Connection reset.",
            Errno::Deadlk => "Resource deadlock would occur.",
            Errno::Destaddrreq => "Destination address required.",
            Errno::Dom => "Mathematics argument out of domain of function.",
            Errno::Dquot => "Reserved.",
            Errno::Exist => "File exists.",
            Errno::Fault => "Bad address.",
            Errno::Fbig => "File too large.",
            Errno::Hostunreach => "Host is unreachable.",
            Errno::Idrm => "Identifier removed.",
            Errno::Ilseq => "Illegal byte sequence.",
            Errno::Inprogress => "Operation in progress.",
            Errno::Intr => "Interrupted function.",
            Errno::Inval => "Invalid argument.",
            Errno::Io => "I/O error.",
            Errno::Isconn => "Socket is connected.",
            Errno::Isdir => "Is a directory.",
            Errno::Loop => "Too many levels of symbolic links.",
            Errno::Mfile => "File descriptor value too large.",
            Errno::Mlink => "Too many links.",
            Errno::Msgsize => "Message too large.",
            Errno::Multihop => "Reserved.",
            Errno::Nametoolong => "Filename too long.",
            Errno::Netdown => "Network is down.",
            Errno::Netreset => "Connection aborted by network.",
            Errno::Netunreach => "Network unreachable.",
            Errno::Nfile => "Too many files open in system.",
            Errno::Nobufs => "No buffer space available.",
            Errno::Nodev => "No such device.",
            Errno::Noent => "No such file or directory.",
            Errno::Noexec => "Executable file format error.",
            Errno::Nolck => "No locks available.",
            Errno::Nolink => "Reserved.",
            Errno::Nomem => "Not enough space.",
            Errno::Nomsg => "No message of the desired type.",
            Errno::Noprotoopt => "Protocol not available.",
            Errno::Nospc => "No space left on device.",
            Errno::Nosys => "Function not supported.",
            Errno::Notconn => "The socket is not connected.",
            Errno::Notdir => "Not a directory or a symbolic link to a directory.",
            Errno::Notempty => "Directory not empty.",
            Errno::Notrecoverable => "State not recoverable.",
            Errno::Notsock => "Not a socket.",
            Errno::Notsup => "Not supported, or operation not supported on socket.",
            Errno::Notty => "Inappropriate I/O control operation.",
            Errno::Nxio => "No such device or address.",
            Errno::Overflow => "Value too large to be stored in data type.",
            Errno::Ownerdead => "Previous owner died.",
            Errno::Perm => "Operation not permitted.",
            Errno::Pipe => "Broken pipe.",
            Errno::Proto => "Protocol error.",
            Errno::Protonosupport => "Protocol not supported.",
            Errno::Prototype => "Protocol wrong type for socket.",
            Errno::Range => "Result too large.",
            Errno::Rofs => "Read-only file system.",
            Errno::Spipe => "Invalid seek.",
            Errno::Srch => "No such process.",
            Errno::Stale => "Reserved.",
            Errno::Timedout => "Connection timed out.",
            Errno::Txtbsy => "Text file busy.",
            Errno::Xdev => "Cross-device link.",
            Errno::Notcapable => "Extension: Capabilities insufficient.",
            Errno::Shutdown => "Cannot send after socket shutdown.",
            Errno::Memviolation => "Memory access violation.",
            Errno::Unknown => "An unknown error has occured",
        }
    }
}