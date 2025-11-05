use core::panic;
use std::mem::transmute;

use log::info;
use as_hostcall::{
    types::{FsImageFunc, IsolationID, MetricEvent, MetricFunc, NetdevName},
    CommonHostCall, HostCallID,
};

#[cfg(feature = "enable_mpk")]
use crate::mpk;
#[cfg(feature = "enable_mpk")]
use as_hostcall::mpk::LIBOS_PKEY;

use crate::{isolation::get_isol, logger};

/// # Safety
/// This is unsafe because it it be a callback function used to lookup the address of
/// hostcall function symbols, and it should be only invocated by service modules.
///
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn find_host_call(isol_id: IsolationID, hc_id: HostCallID) -> usize {
    // let id = HostCallID::Common(CommonHostCall::Write);
    // thread::sleep(Duration::from_secs(1));
    logger::debug!(
        "find_host_call, isol_id={isol_id}, call_id={:?}, call_name={}",
        hc_id,
        hc_id.to_string()
    );
    let isol = get_isol(isol_id).expect("isol don't exist?");

    let addr = match hc_id {
        HostCallID::Common(CommonHostCall::Metric) => metric_handler as MetricFunc as usize,
        HostCallID::Common(CommonHostCall::FsImage) => fs_image_handler as FsImageFunc as usize,
        HostCallID::Common(CommonHostCall::SpawnFaultThread) => spwan_fault_thread_handler as usize,
        _ => {
            let svc_name = hc_id.belong_to();
            logger::debug!(
                "hostcall_{} belong to service: {}",
                hc_id.to_string(),
                svc_name
            );

            let service = isol.service_or_load(&svc_name).unwrap_or_else(|e| {
                panic!("need find: {}, need load: {}, err: {}", hc_id, svc_name, e)
            });

            let symbol = service
                .interface::<fn()>(&hc_id.to_string())
                .unwrap_or_else(|| {
                    panic!(
                        "not found interface \"{}\" in service \"{}\"",
                        hc_id,
                        hc_id.belong_to()
                    )
                });
            *symbol as usize
        }
    };

    log::debug!("interface '{}' addr = 0x{:x}", hc_id, addr);
    addr
}

fn metric_handler(isol_id: IsolationID, event: MetricEvent) -> Result<(), ()> {
    let isol = get_isol(isol_id).expect("isol don't exist?");
    isol.metric.mark(event);

    Ok(())
}

fn fs_image_handler(isol_id: IsolationID) -> Option<String> {
    get_isol(isol_id)
        .expect("isol don't exist?")
        .fs_image
        .clone()
        .map(|image| {
            info!("fs_image_handler: will use image: {}", image);
            image
        })
}

fn spwan_fault_thread_handler(isol_id: IsolationID) -> Result<(), String> {
    info!("enter spwan_fault_thread_handler, isol_id={}", isol_id);

    let isol = get_isol(isol_id).expect("isol don't exist?");
    let mmap_file_backend = isol
        .service_or_load(&"mmap_file_backend".to_owned())
        .map_err(|_| "missing common_service: mmap_file_backend?")?;

    let fault_handler: Option<libloading::Symbol<'_, fn()>> =
        mmap_file_backend.interface(&CommonHostCall::FilePageFaultHandler.to_string());
    let fault_handler_addr = if let Some(fault_handler) = fault_handler {
        *fault_handler as usize
    } else {
        Err(format!(
            "missing inferface: {}",
            CommonHostCall::FilePageFaultHandler
        ))?
    };

    let thread_builder =
        std::thread::Builder::new().name(format!("isol-{}-fault-handler", isol.id));

    // TODO: Save thread handler to send exit signal.
    let _thread_handler = thread_builder
        .spawn(move || {
            let fault_handler: fn() = unsafe { transmute(fault_handler_addr) };
            fault_handler()
        })
        .expect("spawn thread failed.");

    Ok(())
}

#[test]
fn find_host_call_test() {
    // logger::init();
    use crate::{
        isolation::{config::LoadableUnit, Isolation, IsolationConfig},
        utils,
    };

    let isol = {
        // let mut isol_table = ISOL_TABLE.lock().unwrap();
        let services = vec![LoadableUnit(
            "fdtab".to_owned(),
            utils::TARGET_DEBUG_PATH.join("libfdtab.so"),
        )];

        log::debug!("services={:#?}", services);

        Isolation::new(&IsolationConfig {
            services,
            apps: vec![LoadableUnit(
                "hello1".to_owned(),
                utils::TARGET_DEBUG_PATH.join("libhello_world.so"),
            )],
            groups: Default::default(),
            fs_image: None,
            with_libos: None,
        })
        // isol_table.insert(1, Arc::clone(&isol));
    };

    let hostcall_id = HostCallID::Common(as_hostcall::CommonHostCall::Write);
    let addr = unsafe { find_host_call(1, hostcall_id) };

    let fs_svc = isol
        .service_or_load(&"fdtab".to_string())
        .expect("service not found?");

    let symbol = fs_svc
        .interface::<fn()>("write")
        .expect("not found interface 'write'");

    assert!(addr == *symbol as usize)
}

pub fn netdev_alloc_handler() -> Result<NetdevName, ()> {
    Err(())
}

/// A panic handler that should be registered into hostcalls.
///
/// ## TODO
/// This is a bad implementation because had better just make
/// threads of app exit gracefully. So maybe better call `pthread_exit`
/// rather than `panic!`.
///
/// ## Safety
/// It should only be invoked by panic_handler of as_std.
pub unsafe extern "C" fn panic_handler() -> ! {
    panic!()
    // core::panic!()
}
