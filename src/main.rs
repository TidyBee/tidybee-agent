mod options;
mod watcher;

use notify::{RecursiveMode, Watcher};
use notify_debouncer_full::new_debouncer;
use std::{process, time::Duration};

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

fn watch_directories(
    directories: Vec<std::path::PathBuf>,
    file_extensions_args: Option<Vec<String>>,
    file_types_args: Option<String>,
) {
    println!("watch directories: {:?}", directories);

    if let Some(e) = file_extensions_args {
        println!("file extensions: {:?}", e);
    }

    if let Some(t) = file_types_args {
        println!("file types: {:?}", t);
    }

    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer: notify_debouncer_full::Debouncer<
        notify::FsEventWatcher,
        notify_debouncer_full::FileIdMap,
    > = match new_debouncer(Duration::from_secs(2), None, tx) {
        Ok(debouncer) => debouncer,
        Err(err) => {
            eprintln!("Error creating debouncer: {:?}", err);
            return;
        }
    };

    for dir in directories {
        if let Err(err) = debouncer.watcher().watch(&dir, RecursiveMode::Recursive) {
            eprintln!("Error watching directory {:?}: {:?}", dir, err);
        } else {
            debouncer.cache().add_root(&dir, RecursiveMode::Recursive);
        }
    }

    for result in rx {
        match result {
            Ok(events) => events
                .iter()
                .for_each(|event: &notify_debouncer_full::DebouncedEvent| println!("{event:?}")),
            Err(errors) => errors
                .iter()
                .for_each(|error: &notify::Error| println!("{error:?}")),
        }
        println!();
    }
}

fn main() {
    let options: Result<options::Options, options::OptionsError> = options::get_options();

    match options {
        Ok(opts) => {
            if let Some(directories) = opts.directories_list_args {
                list_directories(directories, opts.file_extensions_args, opts.file_types_args);
            } else if let Some(directories) = opts.directories_watch_args {
                watch_directories(directories, opts.file_extensions_args, opts.file_types_args);
            }
        }
        Err(error) => {
            options::print_option_error(error);
            process::exit(1);
        }
    }
}
