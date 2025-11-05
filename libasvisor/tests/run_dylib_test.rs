use std::thread::{self, JoinHandle};

use libasvisor::{
    isolation::{config::IsolationConfig, Isolation},
    logger,
};

#[test]
fn run_dylib_test() {
    // logger::init();
    let config1 =
        IsolationConfig::from_file("base_config.json".into()).expect("Open config file failed.");

    let isol1 = Isolation::new(&config1);
    assert!(isol1.run().is_ok());
}

#[cfg(feature = "namespace")]
#[test]
fn run_multi_dylib_test() {
    logger::init();

    let threads: Vec<JoinHandle<_>> = (0..5).map(|_| thread::spawn(run_dylib_test)).collect();

    for thread in threads {
        thread.join().expect("thread join failed");
    }
}
