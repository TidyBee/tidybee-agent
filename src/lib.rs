mod configuration_wrapper;
mod file_info;
mod http_server;
mod lister;
mod logger;
mod my_files;
mod options_parser;
mod watcher;

use crate::http_server::http_server::HttpServerBuilder;
use log::{debug, error, info};
use std::process;
use std::thread;

pub async fn run() {
    let configuration_wrapper: configuration_wrapper::ConfigurationWrapper =
        configuration_wrapper::ConfigurationWrapper::new().unwrap();
    if logger::init_logger(&configuration_wrapper).is_err() {
        process::exit(1);
    }
    let options: Result<options_parser::Options, options_parser::OptionsError> =
        options_parser::get_options();
    info!("Command-line Arguments Parsed");

    let my_files: my_files::MyFiles = my_files::MyFilesBuilder::new()
        .configuration_wrapper(configuration_wrapper.clone())
        .seal()
        .build()
        .unwrap();
    info!("MyFilesDB sucessfully created");
    my_files.init_db().unwrap();
    info!("MyFilesDB sucessfully initialized");

    match options {
        Ok(opts) => {
            let directories_list_args: Vec<std::path::PathBuf> =
                opts.directories_list_args.unwrap_or_default();
            let directories_watch_args: Vec<std::path::PathBuf> =
                opts.directories_watch_args.unwrap_or_default();
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
            let server = HttpServerBuilder::new()
                .configuration_wrapper(configuration_wrapper)
                .build();
            info!("HTTP Server build");
            info!("Directory Successfully Listed");
            tokio::spawn(async move {
                server.await.start().await;
            });
            info!("HTTP Server Started");

            let (sender, receiver) = crossbeam_channel::unbounded();
            let watch_directories_thread: thread::JoinHandle<()> = thread::spawn(move || {
                watcher::watch_directories(
                    directories_watch_args.clone(),
                    opts.file_extensions_args.clone(),
                    opts.file_types_args.clone(),
                    sender,
                );
            });
            info!("File Events Watcher Started");
            for event in receiver {
                debug!("{:?}", event);
            }

            watch_directories_thread.join().unwrap();
        }
        Err(error) => {
            options_parser::print_option_error(error);
            process::exit(1);
        }
    }
}
