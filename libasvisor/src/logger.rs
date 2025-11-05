pub use log::{debug, error, info, warn};
use std::env;

pub fn init() {
    if  env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "WARN");
    }
    env_logger::init();
}
