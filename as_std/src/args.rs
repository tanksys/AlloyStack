#[derive(Debug)]
pub(crate) struct ArgsItem {
    pub(crate) key: heapless::String<32>,
    pub(crate) val: heapless::String<32>,
}

pub fn get(name: &str) -> Option<&'static str> {
    args_list()
        .iter()
        .find(|item| item.key == name)
        .map(|item| item.val.as_str())
}

pub fn all() -> Vec<(String, String)> {
    args_list()
        .iter()
        .map(|item| {
            (
                String::from(item.key.as_str()),
                String::from(item.val.as_str()),
            )
        })
        .collect()
}

fn args_list() -> &'static heapless::Vec<ArgsItem, 16> {
    let mut args_base_addr: usize;
    unsafe {
        core::arch::asm!(
            "mov {}, rsp", out(reg) args_base_addr
        )
    };
    let page_size = 0x1000;
    let args_base_addr = (args_base_addr + page_size - 1) & (!page_size + 1);
    unsafe { &*(args_base_addr as *const heapless::Vec<ArgsItem, 16>) }
}
use alloc::{string::String, vec::Vec};
