use crate::configuration_wrapper::ConfigurationWrapper;
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;

#[derive(Debug, serde::Deserialize)]
struct LogLevels {
    term: String,
    file: String,
}

fn serialize_log_level(level: &str, default_level: LevelFilter) -> LevelFilter {
    match level.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => default_level,
    }
}

pub fn init_logger(
    config_wrapper: &ConfigurationWrapper,
) -> Result<(), Box<dyn std::error::Error>> {
    match config_wrapper.bind::<LogLevels>("log_level") {
        Ok(log_levels) => {
            let term_level = serialize_log_level(&log_levels.term, LevelFilter::Warn);
            let file_level = serialize_log_level(&log_levels.file, LevelFilter::Info);

            CombinedLogger::init(vec![
                TermLogger::new(
                    term_level,
                    Config::default(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(
                    file_level,
                    Config::default(),
                    File::create("tidybee-agent.log").unwrap(),
                ),
            ])
            .map_err(|e| e.into())
        }
        Err(err) => Err(err.into()),
    }
}
