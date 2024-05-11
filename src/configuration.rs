use crate::error::AgentError;
use config::{Config, File};
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentData {
    pub latest_version: String,
    pub minimal_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileSystemInterfaceConfig {
    pub dir: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GrpcServerConfig {
    pub host: String,
    pub protocol: String,
    pub port: u16,
    pub log_level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HubConfig {
    pub host: String,
    pub port: String,
    pub protocol: String,
    pub auth_path: String,
    pub disconnect_path: String,
    pub connection_attempt_limit: u32,
    pub grpc_server: GrpcServerConfig,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct LoggerConfig {
    pub term_level: String,
    pub file_level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub log_level: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Configuration {
    pub agent_data: AgentData,
    pub filesystem_interface_config: FileSystemInterfaceConfig,
    pub hub_config: HubConfig,
    pub logger_config: LoggerConfig,
    pub server_config: ServerConfig,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            agent_data: AgentData {
                latest_version: String::new(),
                minimal_version: String::new(),
            },
            filesystem_interface_config: FileSystemInterfaceConfig {
                dir: vec![["tests", "assets", "test_folder"].iter().collect()],
            },
            hub_config: HubConfig {
                host: String::from("localhost"),
                port: String::from("7001"),
                protocol: String::from("http"),
                auth_path: String::from("/gateway/auth/AOTH"),
                disconnect_path: String::from("/gateway/auth/AOTH/{agent_id}/disconnect"),
                connection_attempt_limit: 30,
                grpc_server: GrpcServerConfig {
                    host: String::from("localhost"),
                    protocol: String::from("http"),
                    port: 5057,
                    log_level: String::from("info"),
                },
            },
            logger_config: LoggerConfig {
                term_level: String::from("debug"),
                file_level: String::from("warn"),
            },
            server_config: ServerConfig {
                address: String::from("0.0.0.0:8111"),
                log_level: String::from("info"),
            },
        }
    }
}

impl Configuration {
    pub fn init() -> Result<Self, AgentError> {
        let env = env::var("TIDY_ENV").unwrap_or_else(|_| "development".into());

        let mut config_dir = env::current_exe()?;
        config_dir.pop();
        config_dir.push("config");

        let builder = Config::builder()
            .add_source(File::from(Path::new("config/default.json")))
            .add_source(File::with_name(&format!("config/{env}.json")).required(false))
            .build()
            .unwrap();
        Ok(builder.try_deserialize().unwrap_or_default())
    }
}
