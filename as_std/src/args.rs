#[derive(Debug)]
pub(crate) struct ArgsItem {
    pub(crate) key: heapless::String<32>,
    pub(crate) val: heapless::String<32>,
}

pub fn get(name: &str) -> Option<&'static str> {
    let mut args_base_addr: usize;
    unsafe {
        core::arch::asm!(
            "mov {}, rsp", out(reg) args_base_addr
        )
    };
    let page_size = 0x1000;
    let args_base_addr = (args_base_addr + page_size - 1) & (!page_size + 1);
    let args_list = unsafe { &mut *(args_base_addr as *mut heapless::Vec<ArgsItem, 16>) };

    for item in args_list {
        if item.key == name {
            return Some(item.val.as_str());
        }
    }

    None
}
