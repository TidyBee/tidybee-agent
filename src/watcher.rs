use std::path::Path;

pub fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
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
