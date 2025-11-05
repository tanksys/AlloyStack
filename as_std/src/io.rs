use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use as_hostcall::fdtab::FdtabError;

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, FdtabError>;
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, FdtabError> {
        let mut size = 0;
        let mut buffer = [0; 1024];

        loop {
            let s = self.read(&mut buffer)?;
            // println!("read {} bytes, total {} bytes.", s, size);
            if s == 0 {
                break;
            }
            size += s;

            buf.extend_from_slice(&buffer[0..s])
        }

        Ok(size)
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, FdtabError> {
        let mut v_buf = Vec::with_capacity(1024);
        self.read_to_end(&mut v_buf)?;

        *buf = String::from_utf8_lossy(&v_buf).to_string();
        Ok(buf.len())
    }
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize, FdtabError>;

    fn write_all(&mut self, mut buf: &[u8]) -> Result<(), FdtabError> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    panic!("write data failed")
                }
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}
