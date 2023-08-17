mod watcher;

fn main() {
    let args: Vec<String> = std::env::args().collect::<Vec<String>>();
    let num_args: usize = args.len();

    if num_args < 2 {
        print_help();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "--watch" => {
            if num_args < 3 {
                print_help();
                std::process::exit(1);
            }
        }
        "--help" => {
            if num_args == 2 {
                print_help();
                std::process::exit(0);
            } else {
                print_help();
                std::process::exit(1);
            }
        }
        _ => {
            print_help();
            std::process::exit(1);
        }
    }
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let file_to_watch_path: String = std::env::args()
        .nth(2)
        .expect("Argument needs to be a path");

    if let Err(error) = watcher::watch(&file_to_watch_path) {
        log::error!("Error: {:?}", error);
    }
}

fn print_help() {
    eprintln!(
        "Usage:
  --watch <directory> : Watch for changes in the specified directory
  --help              : Print help message and exit 0"
    );
}
