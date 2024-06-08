use crate::configuration::Configuration;
use crate::error::AgentError;
use crate::http::hub::Hub;
use crate::server::ServerBuilder;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::{borrow, env, thread};
use tokio::{sync::mpsc, time};
use tracing::{error, Level};

mod agent_data;
mod agent_uuid;
mod configuration;
mod error;
mod file_info;
mod file_lister;
mod file_watcher;
mod http;
mod server;

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
    let config = match Configuration::init() {
        Ok(config) => config,
        Err(err) => {
            return Err(err);
        }
    };

    let selected_cli_logger_level = CLI_LOGGING_LEVEL
        .get(&config.logger_config.term_level)
        .map_or(Level::INFO, borrow::ToOwned::to_owned);

    match env::var("TIDY_BACKTRACE") {
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

    let mut hub_client = Hub::new(config.hub_config.clone()).unwrap();

    tokio::spawn(async move {
        server.start().await;
    });

    let mut timeout = 5;
    loop {
        time::sleep(time::Duration::from_secs(timeout)).await;
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

    match file_lister::list_directories(config.clone().filesystem_interface_config.dir) {
        Ok(files_vec) => {
            if let Err(err) = hub_client
                .grpc_client
                .send_create_events_once(files_vec)
                .await
            {
                error!("{err}");
            }
        }
        Err(error) => {
            error!("{}", error);
        }
    }

    let (file_watcher_sender, file_watcher_receiver) = mpsc::unbounded_channel();
    let file_watcher_thread: thread::JoinHandle<()> = thread::spawn(move || {
        file_watcher::watch_directories(
            config.filesystem_interface_config.dir.clone(),
            file_watcher_sender,
        );
    });

    if let Err(err) = hub_client
        .grpc_client
        .send_events(file_watcher_receiver)
        .await
    {
        error!("{err}");
    }

    file_watcher_thread.join().unwrap();
    Ok(())
}
