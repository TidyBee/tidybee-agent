mod options;

use std::process;

fn print_option_error(error: options::OptionsError) {
    match error {
        options::OptionsError::ConflictingOptions(e) => {
            eprintln!("Error: {}", e);
        }
        options::OptionsError::InvalidDirectory(e) => {
            eprintln!("Error: {}", e);
        }
        options::OptionsError::InvalidFileType(e) => {
            eprintln!("Error: {}", e);
        }
        options::OptionsError::InvalidFileExtension(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

fn main() {
    let options: Result<options::Options, options::OptionsError> = options::get_options();

    match options {
        Ok(_) => {
        }
        Err(error) => {
            print_option_error(error);
            process::exit(1);
        }
    }
    println!("continue");
}
