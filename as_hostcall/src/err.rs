use derive_more::Display;

#[derive(Debug, Display)]
pub enum LibOSErr {
    Unknown,
    BadArgs,

    BadFileDescriptor,
    NoReadPerm,
    NoWritePerm,

    TcpListenErr,
    PhyWaitErr,
    WrongSockState,

    UFFDErr,
}

pub type LibOSResult<T> = Result<T, LibOSErr>;
