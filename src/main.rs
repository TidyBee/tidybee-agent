mod options;

use std::process;

fn main() {
    let options: Result<options::Options, options::OptionsError> = options::get_options();

    match options {
        Ok(options) => {
            if let Some(e) = &options.file_extensions {
                println!("file extensions: {:?}", e);
            }
            if let Some(t) = &options.file_types {
                println!("file types: {:?}", t);
            }
            if let Some(d) = &options.list_directories {
                println!("directories listing: {:?}", d);
            }
            if let Some(d) = &options.watch_directories {
                println!("directories watching: {:?}", d);
            }
        }
        Err(error) => {
            match error {
                options::OptionsError::ConflictingOptions(e) => {
                    eprintln!("Error: {}", e);
                }
                options::OptionsError::InvalidDirectory(e) => {
                    eprintln!("Error: {}", e);
                }
            }
            process::exit(1);
        }
    }
}
