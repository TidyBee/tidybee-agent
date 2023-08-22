mod options;
mod watcher;

use std::process;

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
}

fn main() {
    let options: Result<options::Options, options::OptionsError> = options::get_options();

    match options {
        Ok(opts) => {
            if let Some(directories) = opts.directories_list_args {
                list_directories(directories, opts.file_extensions_args, opts.file_types_args);
                // print file listing error if match
            } else if let Some(directories) = opts.directories_watch_args {
                watch_directories(directories, opts.file_extensions_args, opts.file_types_args);
                // print file watcher error if match
            }
        }
        Err(error) => {
            options::print_option_error(error);
            process::exit(1);
        }
    }
}
