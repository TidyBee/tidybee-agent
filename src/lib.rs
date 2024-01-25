mod agent_data;
mod configuration;
mod configuration_wrapper;
mod file_info;
mod http_server;
mod lister;
mod logger;
mod my_files;
mod watcher;

use http_server::HttpServerBuilder;
use log::{debug, error, info};
use std::thread;

pub async fn run() {
    let configuration_wrapper: configuration_wrapper::ConfigurationWrapper =
        configuration_wrapper::ConfigurationWrapper::new().unwrap();
    let config = configuration::Configuration::init();
    logger::init(
        config.term_log_level.as_str(),
        config.file_log_level.as_str(),
    );

    let my_files_builder = my_files::MyFilesBuilder::new()
        .configuration_wrapper(configuration_wrapper)
        .seal();

    let my_files: my_files::MyFiles = my_files_builder.build().unwrap();
    info!("MyFilesDB sucessfully created");
    my_files.init_db().unwrap();
    info!("MyFilesDB sucessfully initialized");

    match lister::list_directories(config.directories_list_args) {
        Ok(files_vec) => {
            for file in files_vec.iter() {
                match my_files.add_file_to_db(file) {
                    Ok(_) => {}
                    Err(error) => {
                        error!("{}", error);
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
            config.directories_watch_args.clone(),
            config.http_server_address,
        );
    info!("HTTP Server build");
    info!("Directory Successfully Listed");
    tokio::spawn(async move {
        server.start().await;
    });
    info!("HTTP Server Started");

    let (sender, receiver) = crossbeam_channel::unbounded();
    let watch_directories_thread: thread::JoinHandle<()> = thread::spawn(move || {
        watcher::watch_directories(config.directories_watch_args.clone(), sender);
    });
    info!("File Events Watcher Started");
    for event in receiver {
        debug!("{:?}", event);
    }

    watch_directories_thread.join().unwrap();
}
