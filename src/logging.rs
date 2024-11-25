use log::{Level, LevelFilter, Log, Metadata, Record};

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
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

pub fn init() {
    if let Err(e) = log::set_logger(&LOGGER) {
        println!("set logger failed: {} ?", e);
        return;
    }

    let level = option_env!("LOG").unwrap_or("trace");
    // let level = option_env!("LOG").map(|inner| {
    //     inner.chars().map(|c| {
    //         if c >= 'A' && c <= 'Z' {
    //             (c as u8 + 32) as char
    //         } else {
    //             c
    //         }
    //     }).collect::<Vec<char>>()
    // }).unwrap_or("trace");

    let max_level = match level {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Off,
    };

    log::set_max_level(max_level);
}
