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

use http::hub;
use lazy_static::lazy_static;
use notify::EventKind;
use server::ServerBuilder;
use std::{collections::HashMap, path::PathBuf, thread};
use tidy_algo::TidyAlgo;
use tracing::{debug, error, info, Level};

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

    let mut tidy_algo = TidyAlgo::new();
    let basic_ruleset_path: PathBuf = [r"config", r"rules", r"basic.yml"].iter().collect();
    info!("TidyAlgo successfully created");
    match tidy_algo.load_rules_from_file(&my_files, basic_ruleset_path) {
        Ok(loaded_rules_amt) => info!(
            "TidyAlgo successfully loaded {loaded_rules_amt} rules from config/rules/basic.yml"
        ),
        Err(err) => error!("Failed to load rules into TidyAlgo from config/rules/basic.yml: {err}"),
    };

    let server = ServerBuilder::new()
        .my_files_builder(my_files_builder)
        .inject_global_configuration(config.clone())
        .inject_tidy_rules(tidy_algo.clone())
        .build(
            config.agent_data.latest_version.clone(),
            config.agent_data.minimal_version.clone(),
            config.filesystem_interface_config.dir.clone(),
            config.server_config.address.clone(),
            &config.server_config.log_level,
        );
    info!("Server build");

    let mut hub_client = http::hub::Hub::new(config.hub_config.clone());
    info!("Hub Client Created");

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
        &my_files,
        &tidy_algo,
        &mut hub_client,
    )
    .await;

    update_all_grades(&my_files, &tidy_algo);

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

async fn list_directories(
    directories: Vec<PathBuf>,
    my_files: &my_files::MyFiles,
    tidy_algo: &TidyAlgo,
    hub_client: &mut hub::Hub,
) {
    match file_lister::list_directories(directories) {
        Ok(mut files_vec) => {
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
