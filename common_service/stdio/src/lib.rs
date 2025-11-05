use as_hostcall::types::Size;

#[no_mangle]
pub fn host_stdout(buf: &[u8]) -> Size {
    print!("{}", String::from_utf8_lossy(buf));
    buf.len()
}
