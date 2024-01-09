use std::path::PathBuf;

pub struct Configuration {
    pub term_log_level: String,
    pub file_log_level: String,
    pub directories_list_args: Vec<PathBuf>,
    pub directories_watch_args: Vec<PathBuf>,
}

impl Configuration {
    pub fn default() -> Self {
        Self {
            term_log_level: String::from("debug"),
            file_log_level: String::from("warn"),
            directories_list_args: vec![PathBuf::from("src")],
            directories_watch_args: vec![PathBuf::from("src")],
        }
    }
}
