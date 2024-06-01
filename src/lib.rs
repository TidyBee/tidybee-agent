use http::hub;
use lazy_static::lazy_static;
use server::ServerBuilder;
use std::{collections::HashMap, path::PathBuf, thread};
use tracing::{error, info, Level};

mod agent_data;
mod configuration;
mod error;
mod file_info;
mod file_lister;
mod file_watcher;
mod http;
mod server;

use http::hub;
use lazy_static::lazy_static;
use server::ServerBuilder;
use std::{collections::HashMap, path::PathBuf, thread};
use tracing::{error, info, Level};

use crate::error::AgentError;

lazy_static! {
    static ref CLI_LOGGING_LEVEL: HashMap<String, Level> = {
        let mut m = HashMap::new();
        m.insert("trace".to_owned(), Level::TRACE);
        m.insert("debug".to_owned(), Level::DEBUG);
        m.insert("info".to_owned(), Level::INFO);
        m.insert("warn".to_owned(), Level::WARN);
        m.insert("error".to_owned(), Level::ERROR);
        m
    };
}

pub async fn run() -> Result<(), AgentError> {
    info!("Command-line Arguments Parsed");
    let config = match configuration::Configuration::init() {
        Ok(config) => config,
        Err(err) => {
            return Err(err);
        }
    };

    let selected_cli_logger_level = match CLI_LOGGING_LEVEL.get(&config.logger_config.term_level) {
        Some(level) => level.to_owned(),
        None => Level::INFO,
    };
    match std::env::var("TIDY_BACKTRACE") {
        Ok(env) => {
            if env == "1" {
                tracing_subscriber::fmt()
                    .with_target(true)
                    .with_max_level(selected_cli_logger_level)
                    .pretty()
                    .init();
            }
        }
        Err(_) => {
            tracing_subscriber::fmt()
                .with_target(false)
                .with_max_level(selected_cli_logger_level)
                .compact()
                .init();
        }
    };

    let server = ServerBuilder::new()
        .inject_global_configuration(config.clone())
        .build(
            config.agent_data.latest_version.clone(),
            config.agent_data.minimal_version.clone(),
            config.filesystem_interface_config.dir.clone(),
            config.server_config.address.clone(),
            &config.server_config.log_level,
        );
    info!("Server build");

    let mut hub_client = http::hub::Hub::new(config.hub_config.clone()).unwrap(); // The agent should fail if no communication is possible with the Hub

    tokio::spawn(async move {
        server.start().await;
    });
    info!("Server Started");

    let mut timeout = 5;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(timeout)).await;
        if let Err(err) = hub_client.connect().await {
            error!(
                "Error connecting to the hub: {}, retrying in {}",
                err,
                timeout * 2
            );
        } else {
            break;
        }
        timeout *= 2;
    }

    list_directories(
        config.clone().filesystem_interface_config.dir,
        &mut hub_client,
    )
    .await;

    let (file_watcher_sender, file_watcher_receiver) = tokio::sync::mpsc::unbounded_channel();
    let file_watcher_thread: thread::JoinHandle<()> = thread::spawn(move || {
        file_watcher::watch_directories(
            config.filesystem_interface_config.dir.clone(),
            file_watcher_sender,
        );
    });
    info!("File Events Watcher Started");

    hub_client
        .grpc_client
        .send_events(file_watcher_receiver)
        .await;

    file_watcher_thread.join().unwrap();
    Ok(())
}

async fn list_directories(directories: Vec<PathBuf>, hub_client: &mut hub::Hub) {
    match file_lister::list_directories(directories) {
        Ok(files_vec) => {
            hub_client
                .grpc_client
                .send_create_events_once(files_vec)
                .await;
        }
        Err(error) => {
            error!("{}", error);
        }
    }
}
