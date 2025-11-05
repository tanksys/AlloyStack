use libasvisor::isolation::{config::IsolationConfig, Isolation};

#[test]
fn faasflow_test() {
    // logger::init();
    // let config1 =
    //     IsolationConfig::from_file("multi_apps.json".into()).expect("Open config file failed.");

    // let isol1 = Isolation::new(config1);
    // assert!(isol1.run().is_ok());
    // drop(isol1);

    let config2 = IsolationConfig::from_file("pass_complex_args.json".into())
        .expect("Open config file failed.");

    let isol2 = Isolation::new(&config2);

    isol2.preload(&config2).expect("preload failed");

    assert!(isol2.run().is_ok());
}

#[test]
fn long_chain_test() {
    // use libasvisor::logger;

    // logger::init();

    let config1 =
        IsolationConfig::from_file("long_chain.json".into()).expect("Open config file failed.");

    let isol1 = Isolation::new(&config1);
    isol1.preload(&config1).expect("preload failed");

    assert!(isol1.run().is_ok());
}

#[test]
fn map_reduce_test() {
    let config1 =
        IsolationConfig::from_file("map_reduce.json".into()).expect("Open config file failed.");

    let isol1 = Isolation::new(&config1);
    isol1.preload(&config1).expect("preload failed");

    assert!(isol1.run().is_ok());
}
