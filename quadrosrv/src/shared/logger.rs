use chrono::Utc;
use log::{Metadata, Record};

struct Logger;

static LOGGER: Logger = Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "{} [{}] {} ",
                Utc::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

pub fn init_logger() {
    log::set_logger(&LOGGER)
        .map(|()| {
            log::set_max_level(log::LevelFilter::Info);
        })
        .unwrap();
}
