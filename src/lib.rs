mod agent_data;
mod configuration;
mod error;
mod file_info;
mod file_lister;
mod file_watcher;
mod http;
mod my_files;
mod server;

use lazy_static::lazy_static;
use notify::EventKind;
use server::ServerBuilder;
use std::{collections::HashMap, path::PathBuf, thread};
use tracing::{error, info, Level};

use crate::error::MyError;

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

pub async fn run() -> Result<(), MyError> {
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

    let my_files_builder = my_files::MyFilesBuilder::new()
        .configure(config.clone().my_files_config.clone())
        .seal();

    let my_files: my_files::MyFiles = my_files_builder.build().unwrap();
    info!("MyFilesDB successfully created");
    my_files.init_db().unwrap();
    info!("MyFilesDB successfully initialized");

    list_directories(config.clone().filesystem_interface_config.dir, &my_files);

    let server = ServerBuilder::new()
        .my_files_builder(my_files_builder)
        .inject_global_configuration(config.clone())
        .build(
            config.agent_data.latest_version.clone(),
            config.agent_data.minimal_version.clone(),
            config.filesystem_interface_config.dir.clone(),
            config.server_config.address,
            &config.server_config.log_level,
        );
    info!("Server build");

    let hub_client = http::hub::Hub::new(config.hub_config.clone());
    info!("Hub Client Created");

    tokio::spawn(async move {
        server.start().await;
    });
    info!("Server Started");

    tokio::spawn(async move {
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
    });

    let (file_watcher_sender, file_watcher_receiver) = crossbeam_channel::unbounded();
    let file_watcher_thread: thread::JoinHandle<()> = thread::spawn(move || {
        file_watcher::watch_directories(
            config.filesystem_interface_config.dir.clone(),
            file_watcher_sender,
        );
    });
    info!("File Events Watcher Started");
    for file_watcher_event in file_watcher_receiver {
        handle_file_events(&file_watcher_event, &my_files);
    }

    file_watcher_thread.join().unwrap();
    Ok(())
}

fn list_directories(directories: Vec<PathBuf>, my_files: &my_files::MyFiles) {
    match file_lister::list_directories(directories) {
        Ok(mut files_vec) => {
            for file in &mut files_vec {
                match my_files.add_file_to_db(file) {
                    Ok(_) => {
                        // TODO: Send file to the hub
                    }
                    Err(error) => {
                        error!("{:?}", error);
                    }
                }
            }
        }
        Err(error) => {
            error!("{}", error);
        }
    }
}

fn handle_file_events(event: &notify::Event, my_files: &my_files::MyFiles) {
    info!("event: kind: {:?}\tpaths: {:?}", event.kind, &event.paths);

    if let EventKind::Remove(_) = event.kind {
        match my_files.remove_file_from_db(event.paths[0].clone()) {
            Ok(_) => {}
            Err(error) => {
                error!("{:?}", error);
            }
        }
    } else if let EventKind::Create(_) = event.kind {
        if let Some(file) = file_info::create_file_info(&event.paths[0].clone()) {
            match my_files.add_file_to_db(&file) {
                Ok(_) => {}
                Err(error) => {
                    error!("{:?}", error);
                }
            }
        }
    }
}
