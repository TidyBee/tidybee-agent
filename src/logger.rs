use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;

fn get_enum_level(log_level: &str) -> LevelFilter {
    match log_level.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Off,
    }
}

pub fn init(term_log_level: &str, file_log_level: &str) {
    CombinedLogger::init(vec![
        TermLogger::new(
            get_enum_level(term_log_level),
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            get_enum_level(file_log_level),
            Config::default(),
            File::create("tidybee-agent.log").unwrap(),
        ),
    ])
    .unwrap();
}
