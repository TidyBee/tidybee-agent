mod agent_data;
mod configuration;
mod configuration_wrapper;
mod file_info;
mod file_lister;
mod file_watcher;
mod http_server;
mod my_files;
mod tidy_algo;

use crate::tidy_algo::tidy_algo::TidyAlgo;
use http_server::HttpServerBuilder;
use log::{debug, error, info};
use std::{path::PathBuf, thread};

pub async fn run() {
    match std::env::var("TIDY_BACKTRACE") {
        Ok(env) => {
            if env == "1" {
                tracing_subscriber::fmt().with_target(true).pretty().init();
            }
        }
        Err(_) => {
            tracing_subscriber::fmt()
                .with_target(false)
                .compact()
                .init();
        }
    };

    info!("Command-line Arguments Parsed");
    let config = configuration::Configuration::init();

    let my_files_builder = my_files::MyFilesBuilder::new()
        .configure(config.my_files_config)
        .seal();

    let my_files: my_files::MyFiles = my_files_builder.build().unwrap();
    info!("MyFilesDB sucessfully created");
    my_files.init_db().unwrap();
    info!("MyFilesDB sucessfully initialized");

    let mut tidy_algo = TidyAlgo::new();
    info!("TidyAlgo sucessfully created");
    tidy_algo.load_rules_from_file(&my_files, PathBuf::from("config/rules/basic.yml"));
    info!("TidyAlgo sucessfully loaded rules from config/rules/basic.yml");

    match file_lister::list_directories(config.file_lister_config.dir) {
        Ok(files_vec) => {
            for file in &files_vec {
                match my_files.add_file_to_db(file) {
                    Ok(_) => {}
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

    let server = HttpServerBuilder::new()
        .my_files_builder(my_files_builder)
        .build(
            config.file_watcher_config.dir.clone(),
            config.http_server_config.address,
            config.http_server_logging_level,
        );
    info!("HTTP Server build");
    info!("Directory Successfully Listed");
    tokio::spawn(async move {
        server.start().await;
    });
    info!("HTTP Server Started");

    let (sender, receiver) = crossbeam_channel::unbounded();
    let watch_directories_thread: thread::JoinHandle<()> = thread::spawn(move || {
        file_watcher::watch_directories(config.file_watcher_config.dir.clone(), sender);
    });
    info!("File Events Watcher Started");
    for event in receiver {
        debug!("{:?}", event);
    }

    watch_directories_thread.join().unwrap();
}
