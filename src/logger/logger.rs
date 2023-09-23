use simplelog::*;
use std::fs::File;

fn get_log_level(env_var_name: &str, default_level: LevelFilter) -> LevelFilter {
    match std::env::var(env_var_name) {
        Ok(value) => match value.to_lowercase().as_str() {
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => default_level,
        },
        Err(_) => default_level,
    }
}

pub fn init_logger() {
    let term_log_level = get_log_level("TERM_LOG_LEVEL", LevelFilter::Warn);
    let file_log_level = get_log_level("FILE_LOG_LEVEL", LevelFilter::Info);

    CombinedLogger::init(vec![
        TermLogger::new(
            term_log_level,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            file_log_level,
            Config::default(),
            File::create("tidybee-agent.log").unwrap(),
        ),
    ])
    .unwrap();
}
