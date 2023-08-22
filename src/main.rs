mod options;
mod watcher;

use std::process;
use std::thread;

fn list_directories(
    directories: Vec<std::path::PathBuf>,
    file_extensions_args: Option<Vec<String>>,
    file_types_args: Option<String>,
) {
    println!("list directories: {:?}", directories);

    if let Some(e) = file_extensions_args {
        println!("file extensions: {:?}", e);
    }

    if let Some(t) = file_types_args {
        println!("file types: {:?}", t);
    }
}

fn main() {
    let options: Result<options::Options, options::OptionsError> = options::get_options();

    match options {
        Ok(opts) => {
            if let Some(directories) = opts.directories_list_args {
                list_directories(directories, opts.file_extensions_args, opts.file_types_args);
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
                    println!("{event:?}");
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
