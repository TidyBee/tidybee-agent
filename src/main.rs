mod options;

use std::process;

fn main() {
    let options: Result<options::Options, options::OptionsError> = options::get_options();

    if let Err(error) = options {
        options::print_option_error(error);
        process::exit(1);
    }
}
