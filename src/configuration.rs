use config::{Config, File};
use serde_derive::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct FileLister {
    pub dir: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct FileWatcher {
    pub dir: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct HttpServer {
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct LogLevel {
    pub term: String,
    pub file: String,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub file_lister: FileLister,
    pub file_watcher: FileWatcher,
    pub http_server: HttpServer,
    pub log_level: LogLevel,
}

impl Configuration {
    pub fn init() -> Self {
        let builder = Config::builder()
            .add_source(File::from(Path::new("config/configuration.json")))
            .build()
            .unwrap();
        let config: Configuration = builder.try_deserialize().unwrap();
        config
    }
}
