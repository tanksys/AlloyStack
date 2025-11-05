use std::{env::set_current_dir, path::PathBuf};

use libasvisor::{
    isolation::{config::IsolationConfig, Isolation},
    logger, run_single_isol,
};

const ROOT_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[test]
fn run_should_panic_test() {
    logger::init();

    let config1 = IsolationConfig::from_file("should_panic.json".into())
        .expect("Open config file failed.");

    let isol1 = Isolation::new(&config1);
    assert!(isol1.run().is_err());
}
