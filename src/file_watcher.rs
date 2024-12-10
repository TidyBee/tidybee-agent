// use notify::Watcher;
use notify_debouncer_full::new_debouncer;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time;
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

pub fn watch_directories(
    directories: Vec<PathBuf>,
    sender: UnboundedSender<notify_debouncer_full::DebouncedEvent>,
) {
    let (tx, rx) = mpsc::channel();

    let mut debouncer: notify_debouncer_full::Debouncer<
        notify_debouncer_full::notify::INotifyWatcher,
        notify_debouncer_full::NoCache,
    > = match new_debouncer(time::Duration::from_secs(2), None, tx) {
        Ok(debouncer) => debouncer,
        Err(err) => {
            error!("{:?}", err);
            return;
        }
    };

    for directory in directories {
        let clean_directory = match directory.canonicalize() {
            Ok(clean_directory) => clean_directory,
            Err(err) => {
                error!("error with {:?}: {:?}", directory, err);
                continue;
            }
        };

        if let Err(err) = debouncer.watch(&clean_directory, notify::RecursiveMode::Recursive) {
            error!("{:?}: {:?}", clean_directory, err);
        }
    }

    for result in rx {
        match result {
            Ok(events) => {
                for event in &events {
                    sender.send(event.clone()).unwrap();
                }
            }
            Err(errors) => {
                for error in &errors {
                    error!("{error:?}");
                }
            }
        }
    }
}
