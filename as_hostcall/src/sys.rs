pub type FutexFunc = unsafe extern "C" fn(*mut i32, i32, i32, *const u8, *mut i32, i32) -> isize;
pub type GetTidFunc = extern "C" fn() -> isize;
pub type GetRandomFunc = unsafe extern "C" fn(*mut u8, usize, u32) -> isize;
