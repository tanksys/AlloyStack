use libasvisor::{
    isolation::{config::IsolationConfig, Isolation},
    logger,
};

#[test]
fn wasm_fib_test() {
    let config =
        IsolationConfig::from_file("tinywasm.json".into()).expect("Open config file failed.");

    let isol = Isolation::new(&config);

    assert!(isol.run().is_ok());
}

#[test]
fn wasm_py_test() {
    let config =
        IsolationConfig::from_file("tinywasm_py.json".into()).expect("Open config file failed.");

    let isol = Isolation::new(&config);

    assert!(isol.run().is_ok());
}

#[test]
fn wasm_pass_data_test() {
    logger::init();
    let config = IsolationConfig::from_file("tinywasm_pass_args.json".into())
        .expect("Open config file failed.");

    let isol = Isolation::new(&config);

    assert!(isol.run().is_ok());
}

#[test]
fn wasm_wordcount_test() {
    logger::init();
    let config = IsolationConfig::from_file("tinywasm_wordcount.json".into())
        .expect("Open config file failed.");

    let isol = Isolation::new(&config);

    assert!(isol.run().is_ok());
}