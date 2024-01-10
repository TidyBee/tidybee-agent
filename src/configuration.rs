use std::path::PathBuf;

pub struct Configuration {
    pub term_log_level: String,
    pub file_log_level: String,
    pub http_server_address: String,
    pub directories_list_args: Vec<PathBuf>,
    pub directories_watch_args: Vec<PathBuf>,
}

impl Configuration {
    pub fn default() -> Self {
        Self {
            term_log_level: String::from("debug"),
            file_log_level: String::from("warn"),
            http_server_address: String::from("0.0.0.0:8111"),
            directories_list_args: vec![PathBuf::from("src")],
            directories_watch_args: vec![PathBuf::from("src")],
        }
    }
}
