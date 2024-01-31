use config::{Config, File};
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct MyFilesConfiguration {
    pub db_path: String,
    pub drop_db_on_start: bool,
}

#[derive(Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct Configuration {
    pub term_log_level: String,
    pub file_log_level: String,
    pub http_server_address: String,
    pub http_server_logging_level: String,
    pub directories_list_args: Vec<PathBuf>,
    pub directories_watch_args: Vec<PathBuf>,
    pub my_files_configuration: MyFilesConfiguration,
}

impl Configuration {
    pub fn init() -> Self {
        let config = Config::builder()
            .add_source(File::from(Path::new("config/configuration.json")))
            .build()
            .unwrap();
        let app: Configuration = config.try_deserialize().unwrap();
        app
    }

    #[allow(dead_code)]
    pub fn default() -> Self {
        Self {
            term_log_level: String::from("debug"),
            file_log_level: String::from("warn"),
            http_server_address: String::from("0.0.0.0:8111"),
            http_server_logging_level: String::from("info"),
            directories_list_args: vec![PathBuf::from("src")],
            directories_watch_args: vec![PathBuf::from("src")],
            my_files_configuration: MyFilesConfiguration {
                db_path: String::from("my_files.db"),
                drop_db_on_start: false,
            },
        }
    }
}
