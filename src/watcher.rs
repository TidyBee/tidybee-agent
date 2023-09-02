use notify::Watcher;
use std::time;

pub fn watch_directories(
    directories: Vec<std::path::PathBuf>,
    file_extensions_args: Option<Vec<String>>,
    file_types_args: Option<String>,
    sender: crossbeam_channel::Sender<notify_debouncer_full::DebouncedEvent>,
) {
    println!("watch directories: {:?}", directories);

    if let Some(e) = file_extensions_args {
        println!("file extensions: {:?}", e);
    }

    if let Some(t) = file_types_args {
        println!("file types: {:?}", t);
    }

    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer: notify_debouncer_full::Debouncer<
        notify::RecommendedWatcher,
        notify_debouncer_full::FileIdMap,
    > = match notify_debouncer_full::new_debouncer(time::Duration::from_secs(2), None, tx) {
        Ok(debouncer) => debouncer,
        Err(err) => {
            eprintln!("tidybee: error: {:?}", err);
            return;
        }
    };

    for dir in directories {
        if let Err(err) = debouncer
            .watcher()
            .watch(&dir, notify::RecursiveMode::Recursive)
        {
            eprintln!("tidybee: error: {:?}: {:?}", dir, err);
        } else {
            debouncer
                .cache()
                .add_root(&dir, notify::RecursiveMode::Recursive);
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
                    println!("{error:?}");
                }
            }
        }
    }
}
