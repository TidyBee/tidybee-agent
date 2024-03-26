use lazy_static::lazy_static;
use notify::{event::ModifyKind, EventKind};
use server::ServerBuilder;
use std::{
    collections::HashMap,
    path::PathBuf,
    thread::{self},
};
use tidy_algo::TidyAlgo;
use tracing::{debug, error, info, Level};

use crate::utils::{
    path_to_pretty_path, safe_add_file_to_db, safe_remove_file_from_db, update_all_grades,
    update_file,
};

mod agent_data;
mod configuration;
mod file_info;
mod file_lister;
mod file_watcher;
mod http;
mod my_files;
mod server;
mod tidy_algo;
mod tidy_rules;
mod utils;

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

    list_directories(config.clone().file_lister_config.dir, &my_files, &tidy_algo);
    update_all_grades(&my_files, &tidy_algo);

    let server = ServerBuilder::new()
        .my_files_builder(my_files_builder)
        .inject_global_configuration(config.clone())
        .inject_tidy_rules(tidy_algo.clone())
        .build(
            config.agent_data.latest_version.clone(),
            config.agent_data.minimal_version.clone(),
            config.file_watcher_config.dir.clone(),
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
        match hub_client.connect().await {
            Ok(_) => {
                info!("Connected to the hub");
            }
            Err(err) => {
                error!("Error connecting to the hub: {}", err);
            }
        }
    });

    let (file_watcher_sender, file_watcher_receiver) = crossbeam_channel::unbounded();
    let directories = config.file_watcher_config.dir.clone();
    let file_watcher_thread: thread::JoinHandle<()> = thread::spawn(move || {
        file_watcher::watch_directories(directories, file_watcher_sender);
    });
    info!("File Events Watcher Started");
    for file_watcher_event in file_watcher_receiver {
        handle_file_events(
            &file_watcher_event,
            &my_files,
            &tidy_algo,
            config.file_watcher_config.dir.clone(),
        )
        .await;
    }

    file_watcher_thread.join().unwrap();
}

fn list_directories(config: Vec<PathBuf>, my_files: &my_files::MyFiles, tidy_algo: &TidyAlgo) {
    match file_lister::list_directories(config) {
        Ok(mut files_vec) => {
            for file in &mut files_vec {
                match my_files.add_file_to_db(file) {
                    Ok(_) => {
                        tidy_algo.apply_rules(file, my_files);
                        debug!(
                            "{} TidyScore after all rules applied: {:?}",
                            file.path.display(),
                            file.tidy_score
                        );
                        let file_path = file.path.clone();
                        let _ =
                            my_files.set_tidyscore(file_path, file.tidy_score.as_ref().unwrap());
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

async fn handle_file_events(
    event: &notify::Event,
    my_files: &my_files::MyFiles,
    tidy_algo: &TidyAlgo,
    watched_dirs: Vec<PathBuf>,
) {
    if event.kind.is_remove() {
        info!("File removed: {}", event.paths[0].display());
        //let xoxo = my_files.fetch_duplicated_files(my_files, event.paths[0]);
        safe_remove_file_from_db(event.paths[0].clone(), my_files).await;
    } else if event.kind.is_create() {
        info!("File created: {}", event.paths[0].display());
        let pretty_path = path_to_pretty_path(&event.paths[0], watched_dirs.clone());
        safe_add_file_to_db(&pretty_path, my_files).await;
        if let Some(mut file_info) = file_info::create_file_info(&pretty_path) {
            tidy_algo.apply_rules(&mut file_info, my_files);
            my_files.update_grade(event.paths[0].clone(), tidy_algo);
            let _ = my_files.set_tidyscore(file_info.path, file_info.tidy_score.as_ref().unwrap());
        }
    } else if event.kind.is_modify() {
        match event.kind {
            EventKind::Modify(ModifyKind::Metadata(_)) => {
                info!("Metadata modification: {}", event.paths[0].display());
                update_file(&event.paths[0], my_files, tidy_algo);
            }
            EventKind::Modify(ModifyKind::Name(_)) => {
                info!(
                    "File moved: {}: {}",
                    event.paths[0].display(),
                    event.paths[1].display()
                );
                let new_pretty_path = path_to_pretty_path(&event.paths[1], watched_dirs.clone());
                let _ =
                    my_files.update_file_path(&event.paths[0], &event.paths[1], &new_pretty_path);
            }
            EventKind::Modify(ModifyKind::Data(_)) => {
                info!("File content modified: {}", event.paths[0].display());
                // TODO: Create methods to update each fields in my_files
                update_file(&event.paths[0], my_files, tidy_algo);
            }
            _ => {}
        }
    }
}
