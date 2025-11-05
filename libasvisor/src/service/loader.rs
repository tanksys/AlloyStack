use std::{
    collections::HashMap,
    fmt::Display,
    path::PathBuf,
    sync::{Arc, OnceLock},
};

#[cfg(feature = "enable_mpk")]
use crate::mpk;
#[cfg(feature = "enable_mpk")]
use as_hostcall::mpk::LIBOS_PKEY;

use anyhow::anyhow;
use libloading::Library;
use as_hostcall::types::{IsolationID, MetricEvent, ServiceName};
use nix::libc::Lmid_t;

use crate::{isolation::config::IsolationConfig, metric::MetricBucket, utils};

use super::Service;

#[derive(Default)]
pub struct Namespace(Lmid_t);

impl Namespace {
    pub fn as_lmid_t(&self) -> Lmid_t {
        self.0
    }
}

impl Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct ServiceLoader {
    isol_id: IsolationID,
    registered: HashMap<ServiceName, PathBuf>,
    metric: Arc<MetricBucket>,
    namespace: OnceLock<Namespace>,
    with_libos: bool,
}

impl ServiceLoader {
    pub fn new(isol_id: IsolationID, metric: Arc<MetricBucket>, with_libos: bool) -> Self {
        Self {
            isol_id,
            registered: HashMap::new(),
            namespace: OnceLock::new(),
            metric,
            with_libos,
        }
    }

    pub fn register(mut self, config: &IsolationConfig) -> Self {
        for app in &config.apps {
            self.registered.insert(app.0.clone(), app.1.clone());
        }

        for svc in &config.services {
            self.registered.insert(svc.0.clone(), svc.1.clone());
        }
        self
    }

    fn load(&self, name: &ServiceName, pkey: i32) -> Result<Arc<Service>, anyhow::Error> {
        let lib_path = self
            .registered
            .get(name)
            .ok_or(anyhow!("unregistry library, name={}", name))?;

        let metric = self
            .metric
            .new_svc_metric(name.clone(), lib_path.to_string_lossy().to_string());

        // Service init contains Loading elf library.
        metric.mark(MetricEvent::SvcInit);
        let lib = Arc::from(
            load_dynlib(
                lib_path,
                // &format!(
                //     "target/{}/lib{}.so",
                //     if cfg!(debug_assertions) {
                //         "debug"
                //     } else {
                //         "release"
                //     },
                //     name
                // )
                // .into(),
                self.namespace.get().map(|ns| ns.as_lmid_t()),
            )
            .map_err(|e| anyhow!("load_dynlib faile: {e}"))?,
        );
        let service = Service::new(
            name,
            lib_path.to_str().unwrap(),
            lib,
            metric,
            self.with_libos,
            pkey,
        );
        self.namespace.get_or_init(|| service.namespace());

        service.init(self.isol_id)?;
        Ok(Arc::from(service))
    }

    pub fn load_app(&self, name: &ServiceName) -> Result<Arc<Service>, anyhow::Error> {
        let pkey;
        #[cfg(feature = "enable_mpk")]
        {
            pkey = mpk::must_pkey_alloc();
        }
        #[cfg(not(feature = "enable_mpk"))]
        {
            pkey = 0;
        }
        self.load(name, pkey)
    }

    pub fn load_service(&self, name: &ServiceName) -> Result<Arc<Service>, anyhow::Error> {
        self.metric.mark(MetricEvent::LoadService);
        let pkey;
        #[cfg(feature = "enable_mpk")]
        {
            pkey = LIBOS_PKEY;
        }
        #[cfg(not(feature = "enable_mpk"))]
        {
            pkey = 0;
        }

        self.load(name, pkey)
    }
}

/// Checks for the last byte and avoids allocating if it is zero.
///
/// Non-last null bytes still result in an error.
#[cfg(feature = "namespace")]
pub(crate) fn cstr_cow_from_bytes(
    slice: &[u8],
) -> anyhow::Result<std::borrow::Cow<'_, core::ffi::CStr>> {
    static ZERO: std::os::raw::c_char = 0;
    Ok(match slice.last() {
        // Slice out of 0 elements
        None => unsafe { std::borrow::Cow::Borrowed(std::ffi::CStr::from_ptr(&ZERO)) },
        // Slice with trailing 0
        Some(&0) => std::borrow::Cow::Borrowed(std::ffi::CStr::from_bytes_with_nul(slice)?),
        // Slice with no trailing 0
        Some(_) => std::borrow::Cow::Owned(std::ffi::CString::new(slice)?),
    })
}

#[cfg(feature = "namespace")]
fn do_dlmopen(
    filename: &std::path::Path,
    lmid: Option<Lmid_t>,
) -> anyhow::Result<*mut core::ffi::c_void> {
    use std::{ffi::CStr, os::unix::ffi::OsStrExt};

    use log::info;
    use nix::libc::dlerror;

    let handle = unsafe {
        nix::libc::dlmopen(
            lmid.unwrap_or(nix::libc::LM_ID_NEWLM),
            cstr_cow_from_bytes(filename.as_os_str().as_bytes())?.as_ptr(),
            nix::libc::RTLD_LAZY | nix::libc::RTLD_LOCAL,
        )
    };

    info!(
        "load_dynlib: dlmopen, handle=0x{:x}, filename={:?}",
        handle as usize, filename
    );
    if handle.is_null() || handle as usize == 0 {
        let error = unsafe { dlerror() };
        let err_msg = if error.is_null() {
            anyhow!("unknown dlmopen error")
        } else {
            let message = unsafe { CStr::from_ptr(error) }
                .to_string_lossy()
                .to_string();
            anyhow!("dlmopen error: {message}",)
        };
        return Err(err_msg);
    };
    Ok(handle)
}

fn load_dynlib(filename: &PathBuf, lmid: Option<Lmid_t>) -> anyhow::Result<Library> {
    let filename = if !filename.is_file() {
        utils::REPOS_ROOT_PATH.join(filename)
    } else {
        filename.to_owned()
    };

    if !filename.is_file() {
        return Err(anyhow!(
            "load dynlib failed. filename is invaild: {}",
            filename.to_str().unwrap()
        ));
    }
    let lib = {
        #[cfg(feature = "namespace")]
        // libloading do not supply any method like `from_raw(handle: *mut c_void)`.
        unsafe {
            core::mem::transmute(do_dlmopen(&filename, lmid)?)
        }

        #[cfg(not(feature = "namespace"))]
        {
            log::debug!("do not use dlmopen, lmid={:?} is meaningless", lmid);
            unsafe { Library::new(filename) }?
        }
    };
    anyhow::Ok(lib)
}

#[test]
fn service_drop_test() {
    use crate::{
        metric::MetricBucket,
        service::elf_service::{ElfService, WithLibOSService},
    };
    // std::env::set_var("RUST_LOG", "INFO");
    // logger::init();

    let bucket = MetricBucket::new();
    let path = PathBuf::new().join("target/debug/libsocket.so");

    let lib = Arc::from(load_dynlib(&path, None).unwrap());

    let elf = ELFService::new(
        "socket",
        path.to_str().unwrap(),
        lib,
        bucket.new_svc_metric("socket".to_owned(), path.to_string_lossy().to_string()),
        0,
    );
    let socket = WithLibOSService::new(elf);

    drop(socket)
}

// This test case does not work for now, because dlmopen have not be used
// in service loader.
//
// #[test]
// fn test_load_dynlib() {
//     const TARGET_DIR: &str = concat!(
//         env!("CARGO_MANIFEST_DIR"),
//         "/../target/debug/libhello_world.so"
//     );
//     let filename = PathBuf::from(TARGET_DIR);
//     let lib1 = load_dynlib(&filename).unwrap();
//     let lib2 = load_dynlib(&filename).unwrap();
//     unsafe {
//         let addr1 = (*lib1.get::<fn()>(b"rust_main").unwrap()) as usize;
//         let addr2 = (*lib2.get::<fn()>(b"rust_main").unwrap()) as usize;

//         assert!(addr1 != addr2, "addr1:{:x} == addr2:{:x}", addr1, addr2);
//     }
// }
