// provenance is "出处". `strict_provenance` will
// disable casting usize to *const u8.
// #![feature(strict_provenance)]
#![feature(core_panic)]
#![feature(new_uninit)]
#![allow(clippy::result_unit_err)]

mod hostcalls;

pub mod isolation;
pub mod logger;
mod metric;
pub mod service;
pub mod utils;
use std::sync::Arc;

#[cfg(feature = "enable_mpk")]
pub mod mpk;

pub use metric::MetricOpt;

pub use hostcalls::{GetHandlerFuncSybmol, RustMainFuncSybmol, SetHandlerFuncSybmol};

use isolation::{config::IsolationConfig, Isolation};

pub fn run_single_isol<F>(isol_path: String, init_ops: F)
where
    F: FnOnce(),
{
    init_ops();
    let config = IsolationConfig::from_file(isol_path.into()).expect("Open config file failed.");
    let isol = Isolation::new(&config);
    isol.run().expect("isolation user function error.");

    assert_eq!(
        Arc::strong_count(&isol),
        1,
        "isol rc check failed: isol has strong count={}",
        Arc::strong_count(&isol),
    );

    isol.metric.analyze(&MetricOpt::None);
}
