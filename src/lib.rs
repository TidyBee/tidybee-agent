mod agent_data;
mod configuration;
mod file_info;
mod file_lister;
mod file_watcher;
mod http_server;
mod my_files;
mod tidy_algo;
mod tidy_rules;

use http_server::HttpServerBuilder;
use lazy_static::lazy_static;
use notify::EventKind;
use std::{collections::HashMap, path::PathBuf, thread};
use tidy_algo::TidyAlgo;
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
        .configure(config.my_files_config)
        .seal();

    let my_files: my_files::MyFiles = my_files_builder.build().unwrap();
    info!("MyFilesDB sucessfully created");
    my_files.init_db().unwrap();
    info!("MyFilesDB sucessfully initialized");

    let mut tidy_algo = TidyAlgo::new();
    let basic_ruleset_path: PathBuf = vec![r"config", r"rules", r"basic.yml"].iter().collect();
    info!("TidyAlgo sucessfully created");
    match tidy_algo.load_rules_from_file(&my_files, basic_ruleset_path) {
        Ok(loaded_rules_amt) => info!(
            "TidyAlgo sucessfully loaded {loaded_rules_amt} rules from config/rules/basic.yml"
        ),
        Err(err) => error!("Failed to load rules into TidyAlgo from config/rules/basic.yml: {err}"),
    };

    list_directories(config.file_lister_config.dir, &my_files, &tidy_algo);

    let server = HttpServerBuilder::new()
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

fn list_directories(config: Vec<PathBuf>, my_files: &my_files::MyFiles, tidy_algo: &TidyAlgo) {
    match file_lister::list_directories(config) {
        Ok(mut files_vec) => {
            for file in &mut files_vec {
                match my_files.add_file_to_db(file) {
                    Ok(_) => {
                        tidy_algo.apply_rules(file, &my_files);
                        debug!(
                            "{} TidyScore after all rules applied: {:?}",
                            file.path.display(),
                            file.tidy_score
                        );
                        let file_path = file.path.clone();
                        let _ =
                            my_files.set_tidyscore(file_path, &file.tidy_score.as_ref().unwrap());
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
