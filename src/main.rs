use log::info;
use std::env;

mod watcher;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let file_to_watch_path = env::args().nth(1).expect("Argument 1 needs to be a path");

    info!("Watching {}", file_to_watch_path);
    if let Err(error) = watcher::watch(&file_to_watch_path) {
        log::error!("Error: {:?}", error);
    }
}
