mod lister;
mod options_parser;
mod watcher;
mod http_server;

use std::process;
use std::thread;

#[tokio::main]
async fn main() {
    // Object configuration will be used to do config.get_host / config.get_port
    // and then replace static string host & port
    let server = http_server::Server::new("0.0.0.0".to_string(), "3000".to_string());
    let options: Result<options_parser::Options, options_parser::OptionsError> =
        options_parser::get_options();

    match options {
        Ok(opts) => {
            let directories_list_args: Vec<std::path::PathBuf> =
                opts.directories_list_args.unwrap_or_default();
            let directories_watch_args: Vec<std::path::PathBuf> =
                opts.directories_watch_args.unwrap_or_default();

            match lister::list_directories(directories_list_args) {
                Ok(_files_vec) => {
                    println!("Files: {:?}", _files_vec);
                }
                Err(error) => {
                    eprintln!("tidybee-agent: error: {}", error);
                }
            }

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
                println!("Tidybee-agent: new event: {event:?}");
            }
            watch_directories_thread.join().unwrap();
        }
        Err(error) => {
            options_parser::print_option_error(error);
            process::exit(1);
        }
    }
    server.server_start().await;
}
