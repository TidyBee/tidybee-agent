mod lister;
mod parser;
mod watcher;
mod http_server;

use std::process;
use std::thread;
use crate::parser::json_parser;

#[tokio::main]
async fn main() {
    use tokio::runtime;

    let _http_server_thread: thread::JoinHandle<()> = thread::spawn(move || {
        let host = json_parser::read_value_from_file("config.json", "host".to_string()).unwrap().to_string();
        let port = json_parser::read_value_from_file("config.json", "port".to_string()).unwrap().to_string();

        let rt = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(http_server::server_start(host, port));
    });

    let options: Result<parser::Options, parser::OptionsError> =
        parser::get_options();

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
                println!("tidybee-agent: new event: {event:?}");
            }
            watch_directories_thread.join().unwrap();
        }
        Err(error) => {
            parser::print_option_error(error);
            process::exit(1);
        }
    }
}
