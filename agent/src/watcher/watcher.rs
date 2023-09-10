use notify::Watcher;
use std::time;

pub fn watch_directories(
    directories: Vec<std::path::PathBuf>,
    _file_extensions_args: Option<Vec<String>>,
    _file_types_args: Option<String>,
    sender: crossbeam_channel::Sender<notify_debouncer_full::DebouncedEvent>,
) {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer: notify_debouncer_full::Debouncer<
        notify::RecommendedWatcher,
        notify_debouncer_full::FileIdMap,
    > = match notify_debouncer_full::new_debouncer(time::Duration::from_secs(2), None, tx) {
        Ok(debouncer) => debouncer,
        Err(err) => {
            eprintln!("tidybee-agent: error: {:?}", err);
            return;
        }
    };

    for directory in directories {
        if let Err(err) = debouncer
            .watcher()
            .watch(&directory, notify::RecursiveMode::Recursive)
        {
            eprintln!("tidybee-agent: error: {:?}: {:?}", directory, err);
        } else {
            debouncer
                .cache()
                .add_root(&directory, notify::RecursiveMode::Recursive);
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
                    eprintln!("tidybee-agent: error: {error:?}");
                }
            }
        }
    }
}
