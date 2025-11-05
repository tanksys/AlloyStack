use log::{Metadata, Record};
use as_std::time::{self, SystemTime};

pub struct MyLogger {}

pub const LOGGER: MyLogger = MyLogger {};

pub fn new() -> MyLogger {
    MyLogger {}
}

impl log::Log for MyLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let utc = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .as_micros() as f64
            / 1000.0;

        let msg = format!(
            "{}:{}:{} - {}\n",
            record.level(),
            record.target(),
            utc,
            record.args()
        );

        print!("{msg}");
    }

    fn flush(&self) {}
}
