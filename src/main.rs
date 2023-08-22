mod listing;
mod options;
mod watcher;

use std::process;
use std::thread;

fn main() {
    let options: Result<options::Options, options::OptionsError> = options::get_options();

    match options {
        Ok(opts) => {
            if let Some(directories) = opts.directories_list_args {
                match listing::list_directories(directories) {
                    Ok(files) => {
                        println!("{}", serde_json::to_string_pretty(&files).unwrap());
                    }
                    Err(error) => {
                        eprintln!("tidybee: error: {}", error);
                    }
                }
            } else if let Some(directories) = opts.directories_watch_args {
                let (sender, receiver) = crossbeam_channel::unbounded();
                let watch_directories_thread: thread::JoinHandle<()> = thread::spawn(move || {
                    watcher::watch_directories(
                        directories.clone(),
                        opts.file_extensions_args.clone(),
                        opts.file_types_args.clone(),
                        sender,
                    );
                });
                for event in receiver {
                    println!("new event: {event:?}");
                }
                watch_directories_thread.join().unwrap();
            }
        }
        Err(error) => {
            options::print_option_error(error);
            process::exit(1);
        }
    }
}
