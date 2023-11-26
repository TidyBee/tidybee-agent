mod configuration_wrapper;
mod file_info;
mod http_server;
mod lister;
mod logger;
mod my_files;
mod watcher;

use crate::http_server::http_server::HttpServerBuilder;
use log::{debug, error, info};
use std::path::PathBuf;
use std::process;
use std::thread;

pub async fn run() {
    let configuration_wrapper: configuration_wrapper::ConfigurationWrapper =
        configuration_wrapper::ConfigurationWrapper::new().unwrap();
    if logger::init_logger(&configuration_wrapper).is_err() {
        process::exit(1);
    }
    info!("Command-line Arguments Parsed");
    let server = HttpServerBuilder::new()
        .configuration_wrapper(configuration_wrapper.clone())
        .build();
    info!("HTTP Server build");

    let my_files: my_files::MyFiles = my_files::MyFilesBuilder::new()
        .configuration_wrapper(configuration_wrapper)
        .seal()
        .build()
        .unwrap();
    info!("MyFilesDB sucessfully created");
    my_files.init_db().unwrap();
    info!("MyFilesDB sucessfully initialized");

    let directories_list_args: Vec<PathBuf> = vec![PathBuf::from(".")];
    let directories_watch_args: Vec<PathBuf> = vec![PathBuf::from(".")];
    debug!("directories_list_args = {:?}", directories_list_args);
    debug!("directories_watch_args = {:?}", directories_watch_args);

    match lister::list_directories(directories_list_args) {
        Ok(_files_vec) => {
            for file in _files_vec.iter() {
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
    info!("Directory Successfully Listed");
    tokio::spawn(async move {
        server.start().await;
    });
    info!("HTTP Server Started");

    let (sender, receiver) = crossbeam_channel::unbounded();
    let watch_directories_thread: thread::JoinHandle<()> = thread::spawn(move || {
        watcher::watch_directories(directories_watch_args.clone(), sender);
    });
    info!("File Events Watcher Started");
    for event in receiver {
        debug!("{:?}", event);
    }

    watch_directories_thread.join().unwrap();
}
