mod configuration_wrapper;
mod file_info;
mod http_server;
mod lister;
mod logger;
mod options_parser;
mod watcher;
use axum::{routing::get};
use crate::http_server::routes;

use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::process;
use std::thread;
use crate::http_server::http_server::HttpServerBuilder;

#[derive(Debug, Default, Deserialize, Serialize)]
struct HttpServerConfig {
    host: String,
    port: String,
}

#[tokio::main]
async fn main() {
    let configuration_wrapper: configuration_wrapper::ConfigurationWrapper =
        configuration_wrapper::ConfigurationWrapper::new().unwrap();
    if let Err(_) = logger::init_logger(&configuration_wrapper) {
        process::exit(1);
    }
    let options: Result<options_parser::Options, options_parser::OptionsError> =
        options_parser::get_options();
    info!("Command-line Arguments Parsed");
    let http_server_config: HttpServerConfig = configuration_wrapper
        .bind::<HttpServerConfig>("http_server")
        .unwrap_or_default();
    let server = HttpServerBuilder::new()
        .host(http_server_config.host)
        .port(http_server_config.port)
        .add_route("/", get(routes::hello_world))
        .add_route("/users", get(routes::get_users))
        .add_route("/heaviest_files", get(routes::get_heaviest_files))
        .build();
    info!("HTTP Server build");


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
                    debug!("{:?}", _files_vec);
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
