#![no_std]
pub use as_std;

#[no_mangle]
pub fn main() -> as_std::agent::FaaSFuncResult<()> {
    panic!("should return err")
}
