use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        //metadata.level() <= Level::Info
        true
    }

    fn log(&self, record: &Record) {
        // if self.enabled(record.metadata()) {
        //     println!("{} - {}", record.level(), record.args());
        // }
        let color = match record.level() {
            Level::Trace => 90, // BrightBlack
            Level::Debug => 32, // Green
            Level::Info => 34,  //Blue
            Level::Warn => 93,  //Bright Yello
            Level::Error => 31, //Red
        };

        println!(
            "\u{1B}[{}m[{:>5}] {}\u{1B}[0m",
            color,
            record.level(),
            record.args()
        );
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Trace))
}
