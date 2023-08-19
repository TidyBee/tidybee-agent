mod config;

fn main() {
    let config_result: Result<config::Config, config::ParseError> = config::get_options();

    match config_result {
        Ok(config) => {
            if let Some(e) = &config.file_extensions {
                println!("Extensions: {:?}", e);
            }

            if let Some(t) = &config.file_types {
                println!("Types: {:?}", t);
            }

            if let Some(d) = &config.list_directories {
                println!("List Directories: {:?}", d);
            }

            if let Some(d) = &config.watch_directories {
                println!("Watch Directories: {:?}", d);
            }
        }
        Err(error) => {
            match error {
                config::ParseError::ConflictingArguments(msg) => {
                    eprintln!("Error: {}", msg);
                }
            }
            std::process::exit(1);
        }
    }
}
