mod configuration_wrapper;
mod file_info;
mod http_server;
mod lister;
mod my_files;
mod options_parser;
mod watcher;

use serde::{Deserialize, Serialize};
use std::process;
use std::thread;

#[derive(Debug, Default, Deserialize, Serialize)]
struct HttpServerConfig {
    host: String,
    port: String,
}

#[tokio::main]
async fn main() {
    let configuration_wrapper: configuration_wrapper::ConfigurationWrapper =
        configuration_wrapper::ConfigurationWrapper::new().unwrap(); // unwrap should panic if the config fails to load
    let options: Result<options_parser::Options, options_parser::OptionsError> =
        options_parser::get_options();

    let http_server_config: HttpServerConfig = configuration_wrapper
        .bind::<HttpServerConfig>("http_server")
        .unwrap_or_default();
    let server = http_server::HttpServer::new(http_server_config.host, http_server_config.port);

    let my_files = my_files::MyFiles::new(configuration_wrapper).unwrap();
    my_files.init_db().unwrap();

    match options {
        Ok(opts) => {
            let directories_list_args: Vec<std::path::PathBuf> =
                opts.directories_list_args.unwrap_or_default();
            let directories_watch_args: Vec<std::path::PathBuf> =
                opts.directories_watch_args.unwrap_or_default();

            match lister::list_directories(directories_list_args) {
                Ok(_files_vec) => {
                    for file in _files_vec {
                        match my_files.add_file_to_db(&file) {
                            Ok(_) => { println!("new file inserted in my_file: {:?}", file) }
                            Err(err) => { eprintln!("tidybee-agent: error: {}", err) }
                        };
                    }
                    // println!("Files: {:?}", _files_vec);
                }
                Err(error) => {
                    eprintln!("tidybee-agent: error: {}", error);
                }
            }

            tokio::spawn(async move {
                server.server_start().await;
            });

            let (sender, receiver) = crossbeam_channel::unbounded();
            let watch_directories_thread: thread::JoinHandle<()> = thread::spawn(move || {
                watcher::watch_directories(
                    directories_watch_args.clone(),
                    opts.file_extensions_args.clone(),
                    opts.file_types_args.clone(),
                    sender,
                );
            });
            for event in receiver {
                println!("tidybee-agent: new event: {event:?}");
            }

            watch_directories_thread.join().unwrap();
        }
        Err(error) => {
            options_parser::print_option_error(error);
            process::exit(1);
        }
    }
}
