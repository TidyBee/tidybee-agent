mod agent_data;
mod configuration;
mod error;
mod file_info;
mod file_lister;
mod file_watcher;
mod http;
mod my_files;
mod server;
mod tidy_algo;
mod tidy_rules;

use lazy_static::lazy_static;
use notify::EventKind;
use prost_types::Timestamp;
use server::ServerBuilder;
use std::{collections::HashMap, path::PathBuf, thread};
use tidy_algo::TidyAlgo;
use tokio_stream;
use tonic::transport::Channel;
use tracing::{debug, error, info, Level};

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

use tidybee_events::tidy_bee_events_client::TidyBeeEventsClient;
use tidybee_events::{FileInfoUpdateRequest, FileInfoUpdateResponse};
pub mod tidybee_events {
    tonic::include_proto!("tidybee_events");
}

pub async fn run() {
    info!("Command-line Arguments Parsed");
    let config = configuration::Configuration::init();

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

    let mut tidybee_events_client = TidyBeeEventsClient::connect("http://[::1]:5057")
        .await
        .expect("Failed to connect to TidyBeeEventsClient");

    let response_stream = tokio_stream::iter(vec![FileInfoUpdateRequest {
        pretty_path: "test".to_string(),
        path: "test".to_string(),
        size: Some(0),
        hash: Some("test".to_string()),
        last_modified: Some(Timestamp::from(std::time::SystemTime::now())),
        last_accessed: Some(Timestamp::from(std::time::SystemTime::now())),
        is_new: Some(true),
    }]);
    tidybee_events_client
        .update_file_info(response_stream)
        .await
        .expect("Failed to send file info update");

    let my_files_builder = my_files::MyFilesBuilder::new()
        .configure(config.clone().my_files_config.clone())
        .seal();

    let my_files: my_files::MyFiles = my_files_builder.build().unwrap();
    info!("MyFilesDB successfully created");
    my_files.init_db().unwrap();
    info!("MyFilesDB successfully initialized");

    let mut tidy_algo = TidyAlgo::new();
    let basic_ruleset_path: PathBuf = vec![r"config", r"rules", r"basic.yml"].iter().collect();
    info!("TidyAlgo successfully created");
    match tidy_algo.load_rules_from_file(&my_files, basic_ruleset_path) {
        Ok(loaded_rules_amt) => info!(
            "TidyAlgo successfully loaded {loaded_rules_amt} rules from config/rules/basic.yml"
        ),
        Err(err) => error!("Failed to load rules into TidyAlgo from config/rules/basic.yml: {err}"),
    };

    list_directories(
        config.clone().filesystem_interface_config.dir,
        &my_files,
        &tidy_algo,
        &mut tidybee_events_client,
    ).await;
    update_all_grades(&my_files, &tidy_algo);

    let server = ServerBuilder::new()
        .my_files_builder(my_files_builder)
        .inject_global_configuration(config.clone())
        .inject_tidy_rules(tidy_algo.clone())
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
}

async fn list_directories(
    directories: Vec<PathBuf>,
    my_files: &my_files::MyFiles,
    tidy_algo: &TidyAlgo,
    tidybee_event_client: &mut TidyBeeEventsClient<Channel>,
) {
    match file_lister::list_directories(directories) {
        Ok(files_vec) => {
            let file_info_update_requests = files_vec
                .iter()
                .map(|file| FileInfoUpdateRequest {
                    pretty_path: file.pretty_path.display().to_string(),
                    path: file.path.display().to_string(),
                    size: Some(file.size),
                    hash: Some(file.hash.clone().unwrap()),
                    last_modified: Some(Timestamp::from(file.last_modified)),
                    last_accessed: Some(Timestamp::from(file.last_accessed)),
                    is_new: Some(true),
                })
                .collect::<Vec<FileInfoUpdateRequest>>();
            tidybee_event_client
                .update_file_info(tokio_stream::iter(file_info_update_requests)).await.expect("Failed to send file info update");
        }
        Err(error) => {
            error!("{}", error);
        }
    }
}

fn update_all_grades(my_files: &my_files::MyFiles, tidy_algo: &TidyAlgo) {
    let files = my_files.get_all_files_from_db();
    match files {
        Ok(files) => {
            for file in files {
                let file_path = file.path.clone();
                my_files.update_grade(file_path, tidy_algo);
            }
        }
        Err(error) => {
            error!("{:?}", error);
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
