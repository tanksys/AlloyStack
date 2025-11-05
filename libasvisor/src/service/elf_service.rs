//! `elf_service` use POSIX API `dlopen` to create service.
//! In the future, it will be **discarded**.

use std::{
    collections::{BTreeMap, HashSet},
    ffi::c_void,
    mem::{transmute, MaybeUninit},
    ptr::NonNull,
    str::FromStr,
    sync::Arc,
};

use anyhow::anyhow;
use lazy_static::lazy_static;
use libloading::{Library, Symbol};

use log::info;
use as_hostcall::{
    types::{DropHandlerFunc, IsolationID, MetricEvent, ServiceName},
    IsolationContext, SERVICE_HEAP_SIZE, SERVICE_STACK_SIZE,
};
use nix::libc::{PF_KEY, RTLD_DI_LMID};
use thiserror::Error;

#[cfg(feature = "enable_mpk")]
use crate::mpk;
use crate::{
    isolation::handler::{find_host_call, panic_handler},
    logger,
    metric::SvcMetricBucket,
    utils::PAGE_SIZE,
    GetHandlerFuncSybmol, RustMainFuncSybmol, SetHandlerFuncSybmol,
};

use super::loader::Namespace;
use core::arch::asm;

use as_hostcall::types::RustMainFunc;

use std::mem::forget;

lazy_static! {
    static ref SHOULD_NOT_SET_CONTEXT: Arc<HashSet<ServiceName>> = Arc::from({
        let mut hs = HashSet::new();
        #[cfg(feature = "namespace")]
        {
            hs.insert("libc".to_owned());
        }
        hs.insert("stdio".to_owned());
        hs.insert("time".to_owned());
        hs
    });
}

#[test]
fn test_should_not_set_context() {
    assert!(
        SHOULD_NOT_SET_CONTEXT.contains("stdio"),
        "SHOULD_NOT_SET_CONTEXT do not have 'stdio'"
    )
}

#[repr(C, align(4096))]
struct PageAlignedRegion<const SIZE: usize>([u8; SIZE]);

struct ServiceHeap(Box<MaybeUninit<PageAlignedRegion<SERVICE_HEAP_SIZE>>>);

impl ServiceHeap {
    fn new() -> Self {
        Self(Box::new_uninit())
    }

    fn c_ptr(&self) -> NonNull<c_void> {
        NonNull::new(self.0.as_ptr() as usize as *mut std::ffi::c_void).unwrap()
    }

    #[cfg(feature = "enable_mpk")]
    fn must_mprotect(&self, pkey: i32) {
        use nix::libc;

        let heap_start = self.0.as_ptr() as usize;
        mpk::pkey_mprotect(
            heap_start as *mut c_void,
            SERVICE_HEAP_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            pkey,
        )
        .expect("heap set mpk failed.");

        logger::info!(
            "service heap (0x{:x}, 0x{:x}) set mpk success with pkey {}, right {:?}.",
            heap_start,
            heap_start + SERVICE_HEAP_SIZE,
            pkey,
            libc::PROT_READ | libc::PROT_WRITE
        );
    }
}

#[derive(Debug)]
struct ArgsItem {
    _key: heapless::String<32>,
    _val: heapless::String<32>,
}

impl ArgsItem {
    fn from_kv(k: &str, v: &str) -> Self {
        Self {
            _key: heapless::String::from_str(k).unwrap(),
            _val: heapless::String::from_str(v).unwrap(),
        }
    }
}

pub struct UserStack(Box<MaybeUninit<PageAlignedRegion<SERVICE_STACK_SIZE>>>);

impl UserStack {
    fn new() -> Self {
        let stack = Box::new_uninit();
        log::debug!("stack: 0x{:x}", stack.as_ptr() as usize);
        Self(stack)
    }

    fn top(&self) -> usize {
        self.0.as_ptr() as usize + SERVICE_STACK_SIZE - PAGE_SIZE
    }

    fn write_args(&self, args: &BTreeMap<String, String>) {
        let args_base_addr = self.top();
        let args_list = unsafe { &mut *(args_base_addr as *mut heapless::Vec<ArgsItem, 16>) };
        *args_list = heapless::Vec::new();

        for (k, v) in args.iter() {
            args_list.push(ArgsItem::from_kv(k, v)).unwrap()
        }
    }

    #[cfg(feature = "enable_mpk")]
    fn mprotect(&self, pkey: i32) -> anyhow::Result<()> {
        use nix::libc;

        let stack_start = self.0.as_ptr() as usize;
        mpk::pkey_mprotect(
            stack_start as *mut c_void,
            SERVICE_STACK_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            pkey,
        )?;

        logger::info!(
            "user stack (0x{:x}, 0x{:x}) set mpk success with pkey {}, right {:?}.",
            stack_start,
            stack_start + SERVICE_STACK_SIZE,
            pkey,
            libc::PROT_READ | libc::PROT_WRITE
        );

        Ok(())
    }
}

impl Default for UserStack {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ElfService {
    pub name: String,
    #[allow(dead_code)]
    pub path: String,
    lib: Arc<Library>,
    metric: Arc<SvcMetricBucket>,
    #[cfg(feature = "enable_mpk")]
    pub pkey: i32,
}

impl ElfService {
    pub fn new(
        name: &str,
        path: &str,
        lib: Arc<Library>,
        metric: Arc<SvcMetricBucket>,
        pkey: i32,
    ) -> Self {
        metric.mark(MetricEvent::SvcInit);
        logger::debug!("ELFService::new, name={name}");
        // let pkey = mpk::pkey_alloc();
        Self {
            name: name.to_owned(),
            path: path.to_owned(),
            lib,
            metric,
            #[cfg(feature = "enable_mpk")]
            pkey,
        }
    }

    pub fn symbol<T>(&self, symbol: &str) -> Option<Symbol<T>> {
        unsafe { self.lib.get(symbol.as_bytes()) }.ok()
    }

    pub fn init(&self, _isol_id: IsolationID) -> anyhow::Result<()> {
        Ok(())
    }

    fn invoke_elf_symbol(&self, rust_main: RustMainFunc, stack: UserStack) -> Result<(), String> {
        log::info!(
            "service_{} rust_main={:x} thread_name={}",
            self.name,
            rust_main as usize,
            std::thread::current().name().unwrap()
        );
        let user_stack_top = stack.top();

        #[cfg(feature = "enable_mpk")]
        let ret: Result<(), String> = unsafe {
            // PKRU register, 11(libos) 00(data_buffer) 11 11 .. 00(function) ... 11 00(default)
            let pkru: u32 = mpk::grant_func_perm(0xCFFFFFFC, self.pkey)
                .map_err(|e| format!("can't generate pkru: {e}"))?;

            asm!(
                // 保存 caller-saved 寄存器 rcx, rdx, rsi, rdi, r8, r9, r10, r11
                "sub rsp, 0x90",
                "mov [rsp+8], rcx",
                "mov [rsp+16], rdx",
                "mov [rsp+24], rsi",
                "mov [rsp+32], rdi",
                "mov [rsp+40], r8",
                "mov [rsp+48], r9",
                "mov [rsp+56], r10",
                "mov [rsp+64], r11",
                // 跳板
                "mov r11, r13",
                "mov [r12+8], rsp",
                "mov rsp, r12",
                "xor rcx, rcx",
                "mov rdx, rcx",
                "wrpkru",
                "call r11",
                in("rax") pkru,
                in("r12") (user_stack_top-16),
                in("r13") rust_main,
            );

            // 恢复寄存器
            asm!(
                "mov rsp, [rsp+8]",
                "mov rcx, [rsp+8]",
                "mov rdx, [rsp+16]",
                "mov rsi, [rsp+24]",
                "mov rdi, [rsp+32]",
                "mov r8, [rsp+40]",
                "mov r9, [rsp+48]",
                "mov r10, [rsp+56]",
                "mov r11, [rsp+64]",
                "add rsp, 0x90",
            );

            // 获取返回值
            let return_value_addr: u64;
            asm!("mov {}, rax", out(reg) return_value_addr);
            (*(return_value_addr as *const Result<(), String>)).clone()
        };
        #[cfg(not(feature = "enable_mpk"))]
        let ret: Result<(), String> = unsafe {
            asm!(
                // 保存 caller-saved 寄存器 rax, rcx, rdx, rsi, rdi, r8, r9, r10, r11
                "sub rsp, 0x90",
                "mov [rsp+8], rcx",
                "mov [rsp+16], rdx",
                "mov [rsp+24], rsi",
                "mov [rsp+32], rdi",
                "mov [rsp+40], r8",
                "mov [rsp+48], r9",
                "mov [rsp+56], r10",
                "mov [rsp+64], r11",
                // 跳板
                "mov r11, r13",
                "mov [r12+8], rsp",
                "mov rsp, r12",
                "call r11",
                in("r12") (user_stack_top-16),
                in("r13") rust_main,
            );

            // 复原 rsp 寄存器的值
            asm!("mov rsp, [rsp+8]");

            // 恢复寄存器
            asm!(
                "mov rcx, [rsp+8]",
                "mov rdx, [rsp+16]",
                "mov rsi, [rsp+24]",
                "mov rdi, [rsp+32]",
                "mov r8, [rsp+40]",
                "mov r9, [rsp+48]",
                "mov r10, [rsp+56]",
                "mov r11, [rsp+64]",
                "add rsp, 0x90",
            );
            // 获取返回值
            let return_value_addr: u64;
            asm!("mov {}, rax", out(reg) return_value_addr);
            (*(return_value_addr as *const Result<(), String>)).clone()
        };

        logger::info!("{} complete.", self.name);
        ret.map_err(|e| {
            let err_msg = format!("function {} run failed: {}", self.name, e);
            // forget because String refer to heap of libos modules.
            forget(e);
            err_msg
        })
    }

    pub fn run(&self, args: &BTreeMap<String, String>) -> anyhow::Result<()> {
        self.metric.mark(MetricEvent::SvcRun);
        let rust_main: RustMainFuncSybmol =
            self.symbol("rust_main").ok_or(anyhow!("missing main?"))?;
        let rust_main = unsafe { transmute(*rust_main as usize) };

        let stack = UserStack::new();
        stack.write_args(args);
        #[cfg(feature = "enable_mpk")]
        stack.mprotect(self.pkey)?;

        self.invoke_elf_symbol(rust_main, stack)
            .map_err(|e| anyhow!(e))?;

        self.metric.mark(MetricEvent::SvcEnd);

        Ok(())
    }

    pub fn namespace(&self) -> Namespace {
        // The reason for using this hack, is same to `fn load_dynlib()`, that must
        // get `handle: *mut c_void` to call `dlinfo()`.
        let handle: usize = *unsafe { core::mem::transmute::<&Library, &usize>(self.lib.as_ref()) };
        let mut result: Namespace = Namespace::default();

        let info = &mut result as *mut Namespace as usize;
        unsafe { nix::libc::dlinfo(handle as *mut c_void, RTLD_DI_LMID, info as *mut c_void) };
        info!("service_{} belong to namespace: {}", self.name, result);
        result
    }
}

impl Drop for ElfService {
    fn drop(&mut self) {
        if let Some(drop_fn) = self.symbol::<DropHandlerFunc>("drop") {
            logger::info!("service {} will invoke drop symbol.", self.name);
            unsafe { drop_fn() }
        }
    }
}

#[derive(Error, Debug)]
enum ServiceInitError {
    #[error("set isol context failed")]
    SetIsolCtxErr,
    #[error("missing set_handler_addr?")]
    MissingSetCtx,
    #[error("missing get_handler_addr?")]
    MissingGetCtx,
    #[error("check isol ctx failed.")]
    CtxCheckFailed,
}

pub struct WithLibOSService {
    elf: ElfService,

    heap: ServiceHeap,
}

impl WithLibOSService {
    pub fn new(elf_svc: ElfService) -> Self {
        let heap = ServiceHeap::new();

        #[cfg(feature = "enable_mpk")]
        heap.must_mprotect(elf_svc.pkey);

        Self { elf: elf_svc, heap }
    }

    fn should_set_context(&self) -> bool {
        !SHOULD_NOT_SET_CONTEXT.contains(&self.elf.name.to_owned())
    }

    #[inline]
    pub fn name(&self) -> String {
        self.elf.name.clone()
    }

    pub fn path(&self) -> &str {
        self.elf.path.as_str()
    }

    pub fn init(&self, isol_id: IsolationID) -> anyhow::Result<()> {
        info!("init name={}", self.name());
        let heap_start = self.heap.c_ptr().as_ptr() as usize;
        let mut heap_size = SERVICE_HEAP_SIZE;

        if self.name() == "mm" {
            heap_size /= 2;
            info!("set buffer alloc region");
        }

        let heap_range = (heap_start, heap_start + heap_size);
        logger::debug!(
            "init for service_{}, isol_id={}, find_host_call_addr=0x{:x}, heap_range={:x?}",
            self.elf.name,
            isol_id,
            find_host_call as usize,
            heap_range
        );

        // If this is a common_service that does not dependent on IsolationContext,
        // then directly return. Because it is not a no_std elf, and not have
        // symbols `set_handler_addr` and `get_handler_addr`.
        if !self.should_set_context() {
            return Ok(());
        };

        let isol_ctx = IsolationContext {
            isol_id,
            find_handler: find_host_call as usize,
            panic_handler: panic_handler as usize,
            heap_range,
        };

        let set_handler: SetHandlerFuncSybmol = self
            .symbol("set_handler_addr")
            .ok_or(ServiceInitError::MissingSetCtx)?;

        logger::info!("start set_handler...");
        unsafe { set_handler(&isol_ctx) }.map_err(|_| ServiceInitError::SetIsolCtxErr)?;
        logger::info!("set_handler complete.");

        let get_handler: GetHandlerFuncSybmol = self
            .symbol("get_handler_addr")
            .ok_or(ServiceInitError::MissingGetCtx)?;

        logger::debug!(
            "service_{} get_hander addr=0x{:x}.",
            self.elf.name,
            *get_handler as usize
        );

        if unsafe { get_handler() } != find_host_call as usize {
            Err(ServiceInitError::CtxCheckFailed)?
        }

        Ok(())
    }

    pub fn symbol<T>(&self, symbol: &str) -> Option<Symbol<T>> {
        self.elf.symbol(symbol)
    }

    pub fn run(&self, args: &BTreeMap<String, String>) -> anyhow::Result<()> {
        self.elf.run(args)
    }

    pub fn namespace(&self) -> Namespace {
        self.elf.namespace()
    }

    #[cfg(feature = "enable_mpk")]
    pub fn pkey(&self) -> i32 {
        self.elf.pkey
    }
}
