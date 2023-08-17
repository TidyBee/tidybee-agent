use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;

pub fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher: notify::FsEventWatcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                log::info!("Change: {:?}", event)
            }
            Err(error) => {
                log::error!("Error: {:?}", error)
            }
        }
    }
    Ok(())
}
