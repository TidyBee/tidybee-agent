mod options;

use std::process;

fn main() {
    let options: Result<options::Options, options::OptionsError> = options::get_options();

    match options {
        Ok(_) => {}
        Err(error) => {
            options::print_option_error(error);
            process::exit(1);
        }
    }
    println!("continue");
}
