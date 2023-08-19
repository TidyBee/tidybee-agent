use clap::{App, Arg};

fn main() {
    let matches = App::new("TidyBee")
        .version("0.0.1")
        .author("majent4")
        .about("Watch for file changes and list directories")
        .arg(
            Arg::with_name("extension")
                .short("e")
                .long("extension")
                .value_name("EXTENSIONS")
                .multiple(true)
                .use_delimiter(true)
                .takes_value(true)
                .help("Specify extensions"),
        )
        .arg(
            Arg::with_name("type")
                .short("t")
                .long("type")
                .value_name("TYPES")
                .multiple(true)
                .use_delimiter(true)
                .takes_value(true)
                .help("Specify types"),
        )
        .arg(
            Arg::with_name("list")
                .short("l")
                .long("list")
                .value_name("DIRECTORIES")
                .multiple(true)
                .use_delimiter(true)
                .takes_value(true)
                .required_unless("watch")
                .conflicts_with("watch")
                .help("Specify directories for listing"),
        )
        .arg(
            Arg::with_name("watch")
                .short("w")
                .long("watch")
                .value_name("DIRECTORIES")
                .multiple(true)
                .use_delimiter(true)
                .required_unless("list")
                .conflicts_with("list")
                .takes_value(true)
                .help("Specify directories for watching"),
        )
        .get_matches();

    if matches.is_present("list") && matches.is_present("watch") {
        eprintln!("Error: -l and -w cannot be provided at the same time.");
        std::process::exit(1);
    }

    // Process the rest of the arguments as needed
    if let Some(extensions) = matches.values_of("extension") {
        let e: Vec<_> = extensions.collect();
        println!("Extensions: {:?}", e);
    }

    if let Some(types) = matches.values_of("type") {
        let t: Vec<_> = types.collect();
        println!("Types: {:?}", t);
    }

    if let Some(directories) = matches.values_of("list") {
        let d: Vec<_> = directories.collect();
        println!("List Directories: {:?}", d);
    }

    if let Some(directories) = matches.values_of("watch") {
        let d: Vec<_> = directories.collect();
        println!("Watch Directories: {:?}", d);
    }
}
