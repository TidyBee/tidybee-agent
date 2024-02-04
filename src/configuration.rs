use config::{Config, File};
use serde_derive::{Deserialize, Serialize};
use std::env::var as env_var;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct AgentData {
    pub latest_version: String,
    pub minimal_version: String,
}

#[derive(Debug, Deserialize)]
pub struct FileListerConfig {
    pub dir: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct FileWatcherConfig {
    pub dir: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct HttpServerConfig {
    pub address: String,
    pub log_level: String,
}

#[derive(Debug, Deserialize)]
pub struct LoggerConfig {
    pub term_level: String,
    pub file_level: String,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct MyFilesConfiguration {
    pub db_path: String,
    pub drop_db_on_start: bool,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub agent_data: AgentData,
    pub file_lister_config: FileListerConfig,
    pub file_watcher_config: FileWatcherConfig,
    pub http_server_config: HttpServerConfig,
    pub logger_config: LoggerConfig,
    pub my_files_config: MyFilesConfiguration,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            agent_data: AgentData {
                latest_version: String::new(),
                minimal_version: String::new(),
            },
            file_lister_config: FileListerConfig {
                dir: vec![PathBuf::from("src")],
            },
            file_watcher_config: FileWatcherConfig {
                dir: vec![PathBuf::from("src")],
            },
            http_server_config: HttpServerConfig {
                address: String::from("0.0.0.0:8111"),
                log_level: String::from("info"),
            },
            logger_config: LoggerConfig {
                term_level: String::from("debug"),
                file_level: String::from("warn"),
            },
            my_files_config: MyFilesConfiguration {
                db_path: String::from("my_files.db"),
                drop_db_on_start: false,
            },
        }
    }
}

impl Configuration {
    pub fn init() -> Self {
        let env = env_var("ENV").unwrap_or_else(|_| "development".into());

        let builder = Config::builder()
            .add_source(File::from(Path::new("config/default.json")))
            .add_source(File::with_name(&format!("config/{env}.json")).required(false))
            .build()
            .unwrap();
        let config: Configuration = builder.try_deserialize().unwrap_or_default();
        config
    }
}
