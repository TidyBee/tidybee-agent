use config::{Config, File};
use serde_derive::{Deserialize, Serialize};
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
    pub http_server_logging_level: String,
    pub logger_config: LoggerConfig,
    pub my_files_config: MyFilesConfiguration,
}

impl Configuration {
    pub fn init() -> Self {
        let env = std::env::var("ENV").unwrap_or_else(|_| "development".into());

        let builder = Config::builder()
            .add_source(File::from(Path::new("config/configuration.json")))
            .add_source(File::with_name(&format!("config/{env}.json")).required(false))
            .build()
            .unwrap();
        let config: Configuration = builder.try_deserialize().unwrap();
        config
    }
}
