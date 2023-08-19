mod config;

fn main() {
    let config = config::get_options();

    if config.list_directories.is_some() && config.watch_directories.is_some() {
        eprintln!("Error: -l and -w cannot be provided at the same time.");
        std::process::exit(1);
    }

    if let Some(e) = &config.extensions {
        println!("Extensions: {:?}", e);
    }

    if let Some(t) = &config.types {
        println!("Types: {:?}", t);
    }

    if let Some(d) = &config.list_directories {
        println!("List Directories: {:?}", d);
    }

    if let Some(d) = &config.watch_directories {
        println!("Watch Directories: {:?}", d);
    }
}
