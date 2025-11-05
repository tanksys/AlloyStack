#![allow(warnings)]

extern crate alloc;

use core::mem::forget;

#[cfg(feature = "log")]
use as_std::println;
#[cfg(feature = "log")]
use alloc::format;

use alloc::{borrow::ToOwned, string::{String, ToString}, sync::Arc, vec::Vec};
use hashbrown::HashMap;
use sjlj::{longjmp, JumpBuf};
use spin::{Mutex, MutexGuard};
use wasmtime::Caller;

use as_hostcall::{fdtab::FdtabResult, types::{DirEntry, Fd, OpenFlags, OpenMode, Stat}};
use as_std::{
    libos::libos,
    time::{SystemTime, UNIX_EPOCH},
};
use crate::{types::*, LibosCtx};

pub(crate) struct WasiState {
    pub(crate) args: Vec<String>,
}
lazy_static::lazy_static! {
    static ref WASI_STATE: Mutex<HashMap<String, WasiState>> = Mutex::new( HashMap::new() );
}
fn get_hashmap_wasi_state_mut() -> MutexGuard<'static, HashMap<String, WasiState>> {
    WASI_STATE.lock()
}
fn get_wasi_state<'a>(id: &str, map: &'a MutexGuard<'static, HashMap<String, WasiState>>) -> &'a WasiState {
    let wasi_state = map.get(id).unwrap();
    if wasi_state.args.len() == 0 {
        panic!("WASI_STATE uninit")
    }
    wasi_state
}
pub fn set_wasi_state(id: &str, _args: Vec<String>) {
    let mut map = get_hashmap_wasi_state_mut();
    map.insert(id.to_string(), WasiState { args: _args });
}

lazy_static::lazy_static! {
    static ref FD2PATH: Mutex<HashMap<u32, String>> = Mutex::new( HashMap::new() );
}
fn get_hashmap_fd2path_mut() -> MutexGuard<'static, HashMap<u32, String>> {
    FD2PATH.lock()
}
fn get_fd2path<'a>(fd: u32, map: &'a MutexGuard<'static, HashMap<u32, String>>) -> &'a String {
    let path =  map.get(&fd).unwrap();
    if path.len() == 0 {
        panic!("FD2PATH uninit")
    }
    path
}
fn set_fd2path(fd: u32, path: String) {
    let mut map = get_hashmap_fd2path_mut();
    map.insert(fd, path);
}

lazy_static::lazy_static! {
    pub static ref JMP_BUF_MAP: Mutex<HashMap<String, Arc<JumpBuf>>> = Mutex::new( HashMap::new() );
}

pub fn args_get(mut caller: Caller<'_, LibosCtx>, argv: i32, argv_buf: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into args_get");
        println!("args: argv: {:?}, argv_buf: {:?}", argv, argv_buf);
        println!("[Time] args_get: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    // argv是每个arg在argv_buf中的起始地址的数组的起始地址
    // argv_buf是存arg的buf的起始地址
    // 在buf中存入arg，并以\0结尾 (len需要+1)

    let caller_id = &caller.data().id;
    let map = WASI_STATE.lock();
    let wasi_state = get_wasi_state(&caller_id, &map);
    let args: Vec<&[u8]> = wasi_state
        .args
        .iter()
        .map(|a| a.as_bytes())
        .collect::<Vec<_>>();
    let mut offset: usize = 0;

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    #[cfg(feature = "log")]
    println!("arg_vec len: {}", args.len());
    for (i, arg) in args.iter().enumerate() {
        #[cfg(feature = "log")]
        println!(
            "i: {:?}, offset: {:?}, arg: {:?}, arg_len: {:?}",
            i,
            offset,
            arg,
            arg.len()
        );
        let arg_addr = argv_buf as usize + offset;

        memory.write(&mut caller, argv as usize + i * core::mem::size_of::<u32>(), &(arg_addr as u32).to_ne_bytes()).unwrap();
        memory.write(&mut caller, arg_addr, arg).unwrap();
        memory.write(&mut caller, arg_addr + arg.len(), "\0".as_bytes()).unwrap();

        offset += arg.len() + 1;
    }

    Errno::Success as i32
}

pub fn args_sizes_get(mut caller: Caller<'_, LibosCtx>, argc: i32, argv_buf_size: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into args_sizes_get");
        println!("args: argc: {:?}, argv_buf_size: {:?}", argc, argv_buf_size);
        println!("[Time] args_sizes_get: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    let caller_id = &caller.data().id;
    let map = WASI_STATE.lock();
    let wasi_state = get_wasi_state(&caller_id, &map);
    let argc_val = wasi_state.args.len();
    let argv_buf_size_val: usize = wasi_state.args.iter().map(|v| v.len() + 1).sum();

    #[cfg(feature = "log")]
    println!(
        "argc_val={:?}, argv_buf_size_val: {:?}",
        argc_val, argv_buf_size_val
    );

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    memory.write(&mut caller, argc as usize, &(argc_val as u32).to_ne_bytes()).unwrap();
    memory.write(&mut caller, argv_buf_size as usize, &(argv_buf_size_val as u32).to_ne_bytes()).unwrap();

    Errno::Success as i32
}

pub fn clock_res_get(mut caller: Caller<'_, LibosCtx>, clock_id: i32, resolution: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into clock_res_get");
        println!("args: clock_id: {:?}, resolution: {:?}", clock_id, resolution);
        println!("[Time] clock_res_get: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn clock_time_get(mut caller: Caller<'_, LibosCtx>, clock_id: i32, precision: i64, time: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into clock_time_get");
        println!(
            "args: clock_id: {:?}, precision: {:?}, time: {:?}",
            clock_id, precision, time
        );
        println!("[Time] clock_time_get: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let time_var = SystemTime::now().duration_since(UNIX_EPOCH).as_nanos();
    memory.write(&mut caller, time as usize, &time_var.to_ne_bytes()).unwrap();
    
    Errno::Success as i32
}

pub fn environ_get(mut caller: Caller<'_, LibosCtx>, environ: i32, environ_buf: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into environ_get");
        println!("args: environ: {:?}, environ_buf: {:?}", environ, environ_buf);
        println!("[Time] environ_get: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn environ_sizes_get(mut caller: Caller<'_, LibosCtx>, environ_count: i32, environ_buf_size: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into environ_sizes_get");
        println!(
            "args: environ_count: {:?}, environ_buf_size: {:?}",
            environ_count, environ_buf_size
        );
        println!("[Time] environ_sizes_get: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    let count = 0i32;
    let buf_size = 0i32;
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    memory.write(&mut caller, environ_count as usize, &count.to_ne_bytes()).unwrap();
    memory.write(&mut caller, environ_buf_size as usize, &buf_size.to_ne_bytes()).unwrap();

    Errno::Success as i32
}

pub fn fd_advise(mut caller: Caller<'_, LibosCtx>, fd: i32, offset: i64, len: i64, advice: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_advise");
        println!("args: fd: {:?}, offset: {:?}, len: {:?}, advice: {:?}", fd, offset, len, advice);
        println!("[Time] fd_advise: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }
    
    Errno::Success as i32
}

pub fn fd_close(mut caller: Caller<'_, LibosCtx>, fd: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_close");
        println!("args: fd: {:?}", fd);
        println!("[Time] fd_close: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    libos!(close(fd as u32)).unwrap();
    Errno::Success as i32
}

pub fn fd_datasync(mut caller: Caller<'_, LibosCtx>, fd: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_datasync");
        println!("args: fd: {:?}", fd);
        println!("[Time] fd_datasync: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }
    
    Errno::Success as i32
}

pub fn fd_fdstat_get(mut caller: Caller<'_, LibosCtx>, fd: i32, retptr: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_fdstat_get");
        println!("args: fd: {:?}, retptr: {:?}", fd, retptr);
        println!("[Time] fd_fdstat_get: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let mut fdstat: WasiFdstat = WasiFdstat {
        fs_filetype: 0,
        fs_flags: 0,
        fs_rights_base: 0,
        fs_rights_inheriting: 0,
    };
    match fd as u32 {
        0 => {
            // stdin
            fdstat.fs_filetype = 2; // CharacterDevice
            fdstat.fs_flags = 0;
            fdstat.fs_rights_base = 0xFFFFFFFFFFFFFFFF;
            fdstat.fs_rights_inheriting = 0;
        }
        1 => {
            // stdout
            fdstat.fs_filetype = 2;
            fdstat.fs_flags = 1;
            fdstat.fs_rights_base = 0xFFFFFFFFFFFFFFFF;
            fdstat.fs_rights_inheriting = 0;
        }
        2 => {
            // stderr
            fdstat.fs_filetype = 2;
            fdstat.fs_flags = 1;
            fdstat.fs_rights_base = 0xFFFFFFFFFFFFFFFF;
            fdstat.fs_rights_inheriting = 0;
        }
        3 => {
            // root inode
            fdstat.fs_filetype = 3; // Directory
            fdstat.fs_flags = 0;
            fdstat.fs_rights_base = 0xFFFFFFFFFFFFFFFF;
            fdstat.fs_rights_inheriting = 0xFFFFFFFFFFFFFFFF;
        }
        _ => (),
    }
    // Todo: 从表中寻找fd
    // let msFdStat = table.find(fd);
    // fdstat.fs_filetype = match msFdStat.kind {
    //     0 => 4, // 0 代表File，4代表RegularFile
    //     1 => 3  // 1 代表Dir，3代表Directory
    // };
    // fdstat.fs_flags = msFdStat.flags;
    // fdstat.fs_rights_base = msFdStat.fs_rights_base;
    // fdstat.fs_rights_inheriting = msFdStat.fs_rights_inheriting;
    if fd as u32 > 3 {
        // 假设打开的都是文件
        fdstat.fs_filetype = 4;
        fdstat.fs_flags = 0;
        fdstat.fs_rights_base = 0xFFFFFFFFFFFFFFFF;
        fdstat.fs_rights_inheriting = 0xFFFFFFFFFFFFFFFF;
    }

    let buf = (&fdstat) as *const _ as usize;
    let buf = unsafe {
        core::slice::from_raw_parts(buf as *const u8, core::mem::size_of::<WasiFdstat>())
    };
    memory.write(&mut caller, retptr as usize, buf).unwrap();

    Errno::Success as i32
}

pub fn fd_fdstat_set_flags(mut caller: Caller<'_, LibosCtx>, fd: i32, offset: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_fdstat_set_flags");
        println!("args: fd: {:?}, flag: {:?}", fd as u32, offset as u16);
        println!("[Time] fd_fdstat_set_flags: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn fd_filestat_get(mut caller: Caller<'_, LibosCtx>, fd: i32, buf: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_filestat_get");
        println!("args: fd: {:?}, buf: {:?}", fd, buf);
        println!("[Time] fd_filestat_get: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn fd_filestat_set_size(mut caller: Caller<'_, LibosCtx>, fd: i32, st_size: i64) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_filestat_set_size");
        println!("args: fd: {:?}, st_size: {:?}", fd, st_size);
        println!("[Time] fd_filestat_set_size: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn fd_filestat_set_times(mut caller: Caller<'_, LibosCtx>, fd: i32, st_atim: i64, st_mtim: i64, fst_flags: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_filestat_set_times");
        println!("args: fd: {:?}, st_atim: {:?}, st_mtim: {:?}, fst_flags: {:?}", fd, st_atim, st_mtim, fst_flags);
        println!("[Time] fd_filestat_set_times: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }
    
    Errno::Success as i32
}

pub fn fd_pread(mut caller: Caller<'_, LibosCtx>, fd: i32, iovs: i32, iovs_len: i32, offset: i64, nread: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_pread");
        println!(
            "args: fd: {:?}, iovs: {:?}, iovs_len: {:?}, offset: {:?}, nread: {:?}",
            fd, iovs, iovs_len, offset, nread
        );
        println!("[Time] fd_pread: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn fd_pwrite(mut caller: Caller<'_, LibosCtx>, fd: i32, iovs: i32, iovs_len: i32, offset: i64, nwritten: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_pwrite");
        println!(
            "args: fd: {:?}, iovs: {:?}, iovs_len: {:?}, offset: {:?}, nwritten: {:?}",
            fd , iovs , iovs_len , offset , nwritten 
        );
        println!("[Time] fd_pwrite: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn fd_prestat_get(mut caller: Caller<'_, LibosCtx>, fd: i32, retptr: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_prestat_get");
        println!("args: fd: {:?}, retptr: {:?}", fd, retptr);
        println!("[Time] fd_prestat_get: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    match fd {
        3 => {
            // root inode
            let prestat = WasiPrestatT {
                tag: 0, // tag 应为 0，表示这是一个目录，非0表示unknown
                u: WasiPrestatUt {
                    dir: WasiPrestatDir {
                        dirname_len: "/".len() as u32,
                    },
                },
            };

            let buf = (&prestat) as *const _ as usize;
            let buf = unsafe {
                core::slice::from_raw_parts(buf as *const u8, core::mem::size_of::<WasiPrestatT>())
            };
            memory.write(&mut caller, retptr as usize, buf).unwrap();

            Errno::Success as i32
        }
        // Todo: libos需要维护一个表，从表中找fd，找不到就返回Badf
        _ => {
            #[cfg(feature = "log")]
            println!("[WASI ERR] Errno in fd_prestat_get: Badf");
            Errno::Badf as i32
        }
    }
}

pub fn fd_prestat_dir_name(mut caller: Caller<'_, LibosCtx>, fd: i32, path_addr: i32, path_len: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_prestat_dir_name");
        println!(
            "args: fd: {:?}, path_addr: {:?}, path_len: {:?}",
            fd, path_addr, path_len
        );
        println!("[Time] fd_prestat_dir_name: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    // Todo: 从表中寻找fd
    if fd == 3 {
        let name = "/";
        memory.write(&mut caller, path_addr as usize, name.as_bytes()).unwrap();
        Errno::Success as i32
    } else {
        #[cfg(feature = "log")]
        println!("[WASI ERR] Errno in fd_prestat_dir_name: Overflow");
        Errno::Overflow as i32
    }
}

pub fn fd_read(mut caller: Caller<'_, LibosCtx>, fd: i32, iovs_ptr: i32, iovs_len: i32, retptr: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_read");
        println!(
            "args: fd: {:?}, iovs_ptr: {:?}, iovs_len: {:?}, retptr: {:?}",
            fd, iovs_ptr, iovs_len, retptr
        );
        println!("[Time] fd_read: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let mut read_size: usize = 0;

    for i in 0..iovs_len {
        let offset: usize = iovs_ptr as usize + i as usize * core::mem::size_of::<WasiCiovec>();
        let iovs = memory.data(&caller)
                                    .get(offset..)
                                    .and_then(|s| s.get(..core::mem::size_of::<WasiCiovec>() as usize))
                                    .unwrap();
        let iovs: &WasiCiovec = unsafe { &*(iovs.as_ptr() as *const WasiCiovec) };
        let mut buf = memory.data_mut(&mut caller)
                                    .get_mut(iovs.buf as usize..)
                                    .and_then(|s| s.get_mut(..iovs.buf_len as usize))
                                    .unwrap();
        read_size += libos!(read(fd as u32, &mut buf)).unwrap();
    }

    #[cfg(feature = "log")]
    println!("read_size: {:?}", read_size);
    memory.write(&mut caller, retptr as usize, &(read_size as u32).to_ne_bytes()).unwrap();
    Errno::Success as i32
}

pub fn fd_readdir(mut caller: Caller<'_, LibosCtx>, fd: i32, buf: i32, buf_len: i32, cookie: i64, bufused: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_readdir");
        println!(
            "args: fd: {:?}, buf: {:?}, buf_len: {:?}, cookie: {:?}, bufused: {:?}",
            fd, buf, buf_len, cookie, bufused
        );
        println!("[Time] fd_readdir start: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let map = FD2PATH.lock();
    let path = get_fd2path(fd as u32, &map);

    #[cfg(feature = "log")]
    println!("fd_readdir: fd = {:?}, path = {:?}", fd, path);

    let mut entries: Vec<DirEntry> = libos!(readdir(path)).unwrap();
    entries.push(DirEntry{
        dir_path: ".".to_owned(),
        entry_name: ".".to_owned(),
        entry_type: 4 // Directory
    });
    entries.push(DirEntry{
        dir_path: "..".to_owned(),
        entry_name: "..".to_owned(),
        entry_type: 4 // Directory
    });
    entries.sort_by(|a, b| a.entry_name.cmp(&b.entry_name));

    let mut cur_cookie = cookie as u64;
    let mut cur_buf = buf as u32;
    let mut bufused_len: u32 = 0;
    for item in entries.iter().skip(cookie as usize) {
        cur_cookie += 1;
        let dirent = WasiDirent {
            d_next: cur_cookie,
            d_ino: match item.entry_name.as_str() {
                "." | ".." => 0, // fake ino
                _ => {
                    // let item_fd: u32 = libos!(open(&item.dir_path, OpenFlags::empty(), OpenMode::RD)).unwrap();
                    // let item_stat = libos!(stat(item_fd)).unwrap();
                    // libos!(close(item_fd)).unwrap();
                    // item_stat.st_ino

                    // fake implement
                    0
                }
            },
            d_namelen: item.entry_name.len() as u32,
            d_type: match item.entry_type {
                2 => 2, // CharacterDevice
                4 => 3, // Directory
                8 => 4, // Regular file
                _ => 0  // Unknown
            }
        };
        let dirent = (&dirent) as *const _ as usize;
        let dirent = unsafe {
            core::slice::from_raw_parts(dirent as *const u8, core::mem::size_of::<WasiDirent>())
        };

        if bufused_len + core::mem::size_of::<WasiDirent>() as u32 + item.entry_name.len() as u32 > buf_len as u32 {
            #[cfg(feature = "log")]
            println!("[INFO] fd_readdir: bufused overflow!");
            
            let cap: u32 = buf_len as u32 - bufused_len;
            let part_buf: Vec<u8> = [dirent, item.entry_name.as_bytes()].concat();
            let part_buf: &[u8] = part_buf.get(0..(cap as usize)).unwrap();
            
            #[cfg(feature = "log")]
            println!("[INFO] fd_readdir: cap: {:?}, bufused_len: {:?}, buf_len: {:?}", cap, bufused_len, buf_len);
            
            memory.write(&mut caller, cur_buf as usize, part_buf).unwrap();
            memory.write(&mut caller, bufused as usize, &buf_len.to_ne_bytes()).unwrap();
            forget(entries);
            return Errno::Success as i32
        }

        memory.write(&mut caller, cur_buf as usize, dirent).unwrap();
        cur_buf += core::mem::size_of::<WasiDirent>() as u32;
        bufused_len += core::mem::size_of::<WasiDirent>() as u32;

        memory.write(&mut caller, cur_buf as usize, item.entry_name.as_bytes()).unwrap();
        cur_buf += item.entry_name.len() as u32;
        bufused_len += item.entry_name.len() as u32;
    }
    memory.write(&mut caller, bufused as usize, &bufused_len.to_ne_bytes()).unwrap();
    forget(entries);
    Errno::Success as i32
}

pub fn fd_seek(mut caller: Caller<'_, LibosCtx>, fd: i32, offset: i64, whence: i32, pos: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_seek");
        println!(
            "args: fd: {:?}, offset: {:?}, whence: {:?}, pos: {:?}",
            fd, offset, whence, pos
        );
        println!("[Time] fd_seek: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    // TO BE FIX FOR PY HELLO
    // let fd = args.0 as u32;
    // let offset = args.1;
    // let whence = args.2;
    // let pos = offset as u32;
    // // if whence == 0 {

    // // } else if whence == 1 {

    // // } else if whence == 2{

    // // }

    // libos!(lseek(fd, pos)).unwrap();

    Errno::Success as i32
}

pub fn fd_sync(mut caller: Caller<'_, LibosCtx>, fd: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_sync");
        println!("args: fd: {:?}", fd);
        println!("[Time] fd_sync: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }
    
    Errno::Success as i32
}

pub fn fd_tell(mut caller: Caller<'_, LibosCtx>, fd: i32, offset: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_tell");
        println!("args: fd: {:?}, offset: {:?}", fd, offset);
        println!("[Time] fd_tell: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn fd_write(mut caller: Caller<'_, LibosCtx>, fd: i32, iovs_ptr: i32, iovs_len: i32, retptr: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into fd_write");
        println!(
            "args: fd: {:?}, iovs_ptr: {:?}, iovs_len: {:?}, retptr: {:?}",
            fd, iovs_ptr, iovs_len, retptr
        );
        println!("[Time] fd_write: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }
    
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let mut write_size: usize = 0;

    for i in 0..iovs_len {
        let offset: usize = iovs_ptr as usize + i as usize * core::mem::size_of::<WasiCiovec>();
        let iovs = memory.data(&caller)
                                    .get(offset..)
                                    .and_then(|s| s.get(..core::mem::size_of::<WasiCiovec>() as usize))
                                    .unwrap();
        let iovs: &WasiCiovec = unsafe { &*(iovs.as_ptr() as *const WasiCiovec) };
        let buf = memory.data(&caller)
                                    .get(iovs.buf as usize..)
                                    .and_then(|s| s.get(..iovs.buf_len as usize))
                                    .unwrap();
        write_size += libos!(write(fd as u32, &buf)).unwrap();
    }

    #[cfg(feature = "log")]
    println!("write_size: {:?}", write_size);
    memory.write(&mut caller, retptr as usize, &(write_size as u32).to_ne_bytes()).unwrap();
    Errno::Success as i32
}

pub fn path_create_directory(mut caller: Caller<'_, LibosCtx>, fd: i32, path: i32, path_len: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into path_create_directory");
        println!(
            "args: fd: {:?}, path: {:?}, path_len: {:?}",
            fd, path, path_len
        );
        println!("[Time] path_create_directory: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn path_filestat_get(mut caller: Caller<'_, LibosCtx>, fd: i32, flags: i32, path_ptr: i32, path_len: i32, buf: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into path_filestat_get");
        println!(
            "args: fd: {:?}, flags: {:?}, path_ptr: {:?}, path_len: {:?}, buf: {:?}",
            fd, flags, path_ptr, path_len, buf
        );
        println!("[Time] path_filestat_get: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let mut path: Vec<u8> = Vec::with_capacity(path_len as usize);
    path.resize(path_len as usize, 0);
    memory.read(&caller, path_ptr as usize, &mut path).unwrap();
    let path = String::from_utf8(path).expect("[Err] Not a valid UTF-8 sequence");

    #[cfg(feature = "log")]
    println!("path: {:?}", path);

    let path_fd: FdtabResult<Fd> = libos!(open(&path, OpenFlags::empty(), OpenMode::RD));
    let path_fd = if let Err(_e) = path_fd {
        #[cfg(feature = "log")]
        {
            println!("[WASI ERR] Errno in path_filestat_get: Noent");
            println!("[WASI ERR] path error msg: {:?}", _e);
        }
        
        forget(_e);
        return Errno::Noent as i32;
    } else {
        path_fd.unwrap() as u32
    };

    let ruxstat: Stat = libos!(stat(path_fd)).unwrap();
    libos!(close(path_fd)).unwrap();
    let stat = WasiFilestat{
        dev: ruxstat.st_dev,
        ino: ruxstat.st_ino,
        filetype: match ruxstat.st_mode >> 12 {
            2 => 2, // CharacterDevice
            4 => 3, // Directory
            8 => 4, // Regular file
            _ => 0  // Unknown
        },
        nlink: ruxstat.st_nlink,
        size: ruxstat.st_size as u64,
        atim: ruxstat.st_atime.tv_nsec as u64,
        mtim: ruxstat.st_mtime.tv_nsec as u64 ,
        ctim: ruxstat.st_ctime.tv_nsec as u64,
    };

    let stat = (&stat) as *const _ as usize;
    let stat = unsafe {
        core::slice::from_raw_parts(stat as *const u8, core::mem::size_of::<WasiFilestat>())
    };
    memory.write(&mut caller, buf as usize, stat).unwrap();

    Errno::Success as i32
}

pub fn path_filestat_set_times(mut caller: Caller<'_, LibosCtx>, fd: i32, flags: i32, path: i32, path_len: i32, st_atim: i64, st_mtim: i64, fst_flags: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into path_filestat_set_times");
        println!(
            "args: fd: {:?}, flags: {:?}, path: {:?}, path_len: {:?}, st_atim: {:?}, st_mtim: {:?}, fst_flags: {:?}",
            fd, flags, path, path_len, st_atim, st_mtim, fst_flags
        );
        println!("[Time] path_filestat_set_times: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn path_link(mut caller: Caller<'_, LibosCtx>, old_fd: i32, old_flags: i32, old_path: i32, old_path_len: i32, new_fd: i32, new_path: i32, new_path_len: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into path_link");
        println!("args: old_fd: {:?}, old_flags: {:?}, old_path: {:?}, old_path_len: {:?}, new_fd: {:?}, new_path: {:?}, new_path_len: {:?}", old_fd, old_flags, old_path, old_path_len, new_fd, new_path, new_path_len);
        println!("[Time] path_link: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn path_open(mut caller: Caller<'_, LibosCtx>, fd: i32, dirflags: i32, path_addr: i32, path_len: i32, oflags: i32, fs_rights_base: i64, fs_rights_inheriting: i64, fdflags: i32, retptr: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into path_open");
        println!("args: fd: {:?}, dirflags: {:?}, path_addr: {:?}, path_len: {:?}, oflags: {:?}, fs_rights_base: {:?}, fs_rights_inheriting: {:?}, fdflags: {:?}, retptr: {:?}", fd as u32, dirflags as u32, path_addr as u32, path_len as u32, oflags as u16, format!("{:064b}", fs_rights_base as u64), format!("{:064b}", fs_rights_inheriting as u64), fdflags as u16, retptr as u32);
        println!("[Time] path_open: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let mut path: Vec<u8> = Vec::with_capacity(path_len as usize);
    path.resize(path_len as usize, 0);
    memory.read(&caller, path_addr as usize, &mut path).unwrap();
    let path = String::from_utf8(path).expect("[Err] Not a valid UTF-8 sequence");
    #[cfg(feature = "log")]
    println!("path: {:?}", path);

    let mut flags: OpenFlags = OpenFlags::empty();
    if (fdflags as u16 & 1) == 1 {
        flags |= OpenFlags::O_APPEND;
    }
    if (oflags as u16 & 1) == 1 {
        flags |= OpenFlags::O_CREAT;
    }

    let mut mode: OpenMode = OpenMode::empty();
    if ((fs_rights_base as u64 >> 1) & 1) == 1 {
        mode |= OpenMode::RD;
    }
    if ((fs_rights_base as u64 >> 6) & 1) == 1 {
        mode |= OpenMode::WR;
    }

    // Todo: 将对应信息添加到table中，结构体struct {name, fs_flags, fs_rights_base, fs_rights_inheriting, kind}
    // 其中，name是路径，fs_flags是Fdflags类型，fs_rights_base和fs_rights_inheriting直接存输入参数。
    // kind表示文件类型，是个enum。eg. File Dir
    let path_fd: FdtabResult<Fd> = libos!(open(&path, flags, mode));
    let path_fd = if let Err(_e) = path_fd {
        #[cfg(feature = "log")] {
            println!("[WASI ERR] Errno in path_open: Noent");
            println!("[WASI ERR] path error msg: {:?}", _e);
        }

        forget(_e);
        return Errno::Noent as i32;
    } else {
        path_fd.unwrap() as u32
    };

    #[cfg(feature = "log")]
    println!("return path_fd: {:?}", path_fd);
    memory.write(&mut caller, retptr as usize, &path_fd.to_ne_bytes()).unwrap();
    // set fd2path table
    set_fd2path(path_fd as u32, path);
    Errno::Success as i32
}

pub fn path_readlink(mut caller: Caller<'_, LibosCtx>, dir_fd: i32, path: i32, path_len: i32, buf: i32, buf_len: i32, buf_used: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into path_readlink");
        println!("args: dir_fd: {:?}, path: {:?}, path_len: {:?}, buf: {:?}, buf_len: {:?}, buf_used: {:?}", dir_fd, path, path_len, buf, buf_len, buf_used);
        println!("[Time] path_readlink: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }
    
    Errno::Success as i32
}

pub fn path_remove_directory(mut caller: Caller<'_, LibosCtx>, fd: i32, path: i32, path_len: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into path_remove_directory");
        println!(
            "args: fd: {:?}, path: {:?}, path_len: {:?}",
            fd, path, path_len
        );
        println!("[Time] path_remove_directory: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn path_rename(mut caller: Caller<'_, LibosCtx>, old_fd: i32, old_path: i32, old_path_len: i32, new_fd: i32, new_path: i32, new_path_len: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into path_rename");
        println!("args: old_fd: {:?}, old_path: {:?}, old_path_len: {:?}, new_fd: {:?}, new_path: {:?}, new_path_len: {:?}", old_fd, old_path, old_path_len, new_fd, new_path, new_path_len);
        println!("[Time] path_rename: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn path_symlink(mut caller: Caller<'_, LibosCtx>, old_path: i32, old_path_len: i32, fd: i32, new_path: i32, new_path_len: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into path_symlink");
        println!(
            "args: old_path: {:?}, old_path_len: {:?}, fd: {:?}, new_path: {:?}, new_path_len: {:?}",
            old_path, old_path_len, fd, new_path, new_path_len
        );
        println!("[Time] path_symlink: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn path_unlink_file(mut caller: Caller<'_, LibosCtx>, fd: i32, path: i32, path_len: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into path_unlink_file");
        println!(
            "args: fd: {:?}, path: {:?}, path_len: {:?}",
            fd, path, path_len
        );
        println!("[Time] path_unlink_file: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn poll_oneoff(mut caller: Caller<'_, LibosCtx>, in_: i32, out_: i32, nsubscriptions: i32, nevents: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into poll_oneoff");
        println!(
            "args: in_: {:?}, out_: {:?}, nsubscriptions: {:?}, nevents: {:?}",
            in_, out_, nsubscriptions, nevents
        );
        println!("[Time] poll_oneoff: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn proc_exit(mut caller: Caller<'_, LibosCtx>, code: i32) {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into proc_exit");
        // An exit code of 0 indicates successful termination of the program.
        println!("args: code: {:?}", code);
        println!("[Time] proc_exit: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }
    
    match code {
        0 => {
            let caller_id = &caller.data().id;
            let jmpbuf = {
                Arc::clone(JMP_BUF_MAP.lock().get(caller_id).unwrap())
            };
            unsafe { longjmp(jmpbuf.as_ref(), 1); };
        },
        _ => { panic!("[ERR] proc_exit got error code {:?}", code); }
    }
}

pub fn random_get(mut caller: Caller<'_, LibosCtx>, buf: i32, buf_len: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into random_get");
        println!("args: buf: {:?}, buf_len: {:?}", buf, buf_len);
        println!("[Time] random_get: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let array: Vec<u8> = {
        // LCG算法
        let mut seed = buf as u64;
        let mut next_u8 = || {
            const A: u64 = 1664525;
            const C: u64 = 1013904223;
            const MOD: u64 = 1 << 32;
            seed = (A.wrapping_mul(seed).wrapping_add(C)) % MOD;
    
            (seed % 256) as u8
        };

        (0..buf_len as usize).map(|_| next_u8()).collect()
    };

    #[cfg(feature = "log")]
    println!("array: {:?}", array);
    memory.write(&mut caller, buf as usize, &array).unwrap();

    Errno::Success as i32
}

pub fn sched_yield(mut caller: Caller<'_, LibosCtx>) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into sched_yield");
        println!("[Time] sched_yield: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn sock_accept(mut caller: Caller<'_, LibosCtx>, sock: i32, fd_flags: i32, ro_fd: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into sock_accept");
        println!(
            "args: sock: {:?}, fd_flags: {:?}, ro_fd: {:?}",
            sock, fd_flags, ro_fd
        );
        println!("[Time] sock_accept: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn sock_recv(mut caller: Caller<'_, LibosCtx>, sock: i32, ri_data: i32, ri_data_len: i32, ri_flags: i32, ro_data_len: i32, ro_flags: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into sock_recv");
        println!("args: sock: {:?}, ri_data: {:?}, ri_data_len: {:?}, ri_flags: {:?}, ro_data_len: {:?}, ro_flags: {:?}", sock, ri_data, ri_data_len, ri_flags, ro_data_len, ro_flags);
        println!("[Time] sock_recv: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn sock_send(mut caller: Caller<'_, LibosCtx>, sock: i32, si_data: i32, si_data_len: i32, si_flags: i32, ret_data_len: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into sock_send");
        println!(
            "args: sock: {:?}, si_data: {:?}, si_data_len: {:?}, si_flags: {:?}, ret_data_len: {:?}",
            sock, si_data, si_data_len, si_flags, ret_data_len
        );
        println!("[Time] sock_send: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}

pub fn sock_shutdown(mut caller: Caller<'_, LibosCtx>, sock: i32, how: i32) -> i32 {
    #[cfg(feature = "log")]
    {
        println!("[Debug] Invoke into sock_shutdown");
        println!("args: sock: {:?}, how: {:?}", sock, how);
        println!("[Time] sock_shutdown: {}", SystemTime::now().duration_since(UNIX_EPOCH).as_micros() as f64 / 1000000f64);
    }

    Errno::Success as i32
}
