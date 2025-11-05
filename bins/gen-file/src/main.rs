use std::path::PathBuf;

use libasvisor::isolation::config::{
    App, IsolationConfig, IsolationGroup, IsolationGroupApp, LoadableUnit,
};

fn main() {
    let group = IsolationGroup {
        list: vec![IsolationGroupApp::Detailed(App {
            name: "hello1".to_owned(),
            args: Default::default(),
        })],
        args: Default::default(),
    };

    let config1 = IsolationConfig {
        services: vec![
            LoadableUnit(
                "fdtab".to_owned(),
                PathBuf::from("target/debug/libfdtab.so"),
            ),
            LoadableUnit(
                "stdio".to_owned(),
                PathBuf::from("target/debug/libstdio.so"),
            ),
        ],
        apps: vec![LoadableUnit(
            "hello1".to_owned(),
            PathBuf::from("target/debug/libhello_world.so"),
        )],
        groups: vec![group],
        fs_image: Some("fs_images/fatfs.img".to_owned()),
        with_libos: None,
    };

    config1
        .to_file("./config.json".into())
        .expect("Write to file failed.")
}
