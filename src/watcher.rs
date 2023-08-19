use std::path;
use std::sync;

pub fn watch_directory<P: AsRef<path::Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = sync::mpsc::channel();
    let mut watcher: notify::FsEventWatcher = notify::RecommendedWatcher::new(tx, notify::Config::default())?;
    watcher.watch(path.as_ref(), notify::RecursiveMode::Recursive)?;

    for e in rx {
        match e {
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
