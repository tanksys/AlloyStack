use std::path::PathBuf;

use libasvisor::{logger, run_single_isol};

#[test]
fn run_operate_file_test() {
    if !PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fs_images")
        .join("fatfs.img")
        .is_file()
    {
        log::warn!("fs image is not exists. ignore file_test");
        return;
    }

    run_single_isol("simple_file.json".to_owned(), || {
        logger::init();
    });
}

#[test]
fn run_mmap_file_test() {
    if std::env::var("SUDO_PASSWD").is_err() {
        return;
    }

    run_single_isol("mmap_file.json".to_owned(), || {});
}
