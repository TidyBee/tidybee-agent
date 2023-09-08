mod lister;
mod options_parser;
mod watcher;

use std::process;
use std::thread;
//use tokio::io::AsyncWriteExt;
//use tokio::net::TcpStream;

//#[tokio::main]
fn main() {
    //async fn main() {
    //    let mut stream: TcpStream = match TcpStream::connect("localhost:8080").await {
    //        Ok(stream) => stream,
    //        Err(e) => {
    //            eprintln!("tidybee: error: {}", e);
    //            process::exit(1);
    //        }
    //    };

    //    if let Err(e) = send_this(&mut stream, "hiiii").await {
    //        eprintln!("tidybee: error: {}", e);
    //    }

    let options: Result<options_parser::Options, options_parser::OptionsError> =
        options_parser::get_options();

    match options {
        Ok(opts) => {
            if let Some(directories) = opts.directories_list_args {
                match lister::list_directories(directories) {
                    Ok(files) => {
                        //                        let json_data: String = serde_json::to_string_pretty(&files).unwrap();
                        //                        if let Err(e) = send_this(&mut stream, &json_data).await {
                        //                            eprintln!("tidybee: error: {}", e);
                        //                        }
                    }
                    Err(error) => {
                        eprintln!("tidybee: error: {}", error);
                    }
                }
            } else if let Some(directories) = opts.directories_watch_args {
                let (sender, receiver) = crossbeam_channel::unbounded();
                let watch_directories_thread: thread::JoinHandle<()> = thread::spawn(move || {
                    watcher::watch_directories(
                        directories.clone(),
                        opts.file_extensions_args.clone(),
                        opts.file_types_args.clone(),
                        sender,
                    );
                });
                for event in receiver {
                    println!("new event: {event:?}");
                }
                watch_directories_thread.join().unwrap();
            }
        }
        Err(error) => {
            options_parser::print_option_error(error);
            process::exit(1);
        }
    }

    //    if let Err(e) = send_this(&mut stream, "byeee").await {
    //        eprintln!("tidybee: error: {}", e);
    //    }
}

//async fn send_this(
//    stream: &mut TcpStream,
//    message: &str,
//) -> Result<(), Box<dyn std::error::Error>> {
//    stream.write_all(message.as_bytes()).await?;
//    stream.flush().await?;
//    Ok(())
//}
