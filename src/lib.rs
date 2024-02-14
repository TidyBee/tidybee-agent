mod agent_data;
mod configuration;
mod file_info;
mod file_lister;
mod file_watcher;
mod server;
mod my_files;
mod tidy_algo;
mod http;

use crate::tidy_algo::tidy_algo::TidyAlgo;
use server::ServerBuilder;
use notify::EventKind;
use std::{path::PathBuf, thread};
use tracing::{error, info};

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
    let basic_ruleset_path: PathBuf = [r"config", r"rules", r"basic.yml"].iter().collect();
    info!("TidyAlgo sucessfully created");
    tidy_algo.load_rules_from_file(&my_files, basic_ruleset_path);
    info!("TidyAlgo sucessfully loaded rules from config/rules/basic.yml");

    list_directories(config.file_lister_config.dir, &my_files);

    let server = ServerBuilder::new()
        .my_files_builder(my_files_builder)
        .build(
            config.agent_data.latest_version.clone(),
            config.agent_data.minimal_version.clone(),
            config.file_watcher_config.dir.clone(),
            config.http_server_config.address,
            config.http_server_config.log_level,
        );
    info!("HTTP Server build");
    info!("Directory Successfully Listed");
    tokio::spawn(async move {
        server.start().await;
    });
    info!("HTTP Server Started");

    let (file_watcher_sender, file_watcher_receiver) = crossbeam_channel::unbounded();
    let file_watcher_thread: thread::JoinHandle<()> = thread::spawn(move || {
        file_watcher::watch_directories(
            config.file_watcher_config.dir.clone(),
            file_watcher_sender,
        );
    });
    info!("File Events Watcher Started");
    for file_watcher_event in file_watcher_receiver {
        handle_file_events(&file_watcher_event, &my_files);
    }

    file_watcher_thread.join().unwrap();
}

fn list_directories(config: Vec<PathBuf>, my_files: &my_files::MyFiles) {
    match file_lister::list_directories(config) {
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
