use clap::{App, Arg};
use std::path::PathBuf;

pub struct Config {
    pub extensions: Option<Vec<String>>,
    pub types: Option<Vec<String>>,
    pub list_directories: Option<Vec<PathBuf>>,
    pub watch_directories: Option<Vec<PathBuf>>,
}

pub fn get_options() -> Config {
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

    let extensions = matches
        .values_of("extension")
        .map(|exts| exts.map(String::from)
        .collect());
    let types = matches
        .values_of("type")
        .map(|types| types.map(String::from)
        .collect());
    let list_directories = matches
        .values_of("list")
        .map(|dirs| dirs.map(PathBuf::from)
        .collect());
    let watch_directories = matches
        .values_of("watch")
        .map(|dirs| dirs.map(PathBuf::from)
        .collect());
    Config {
        extensions,
        types,
        list_directories,
        watch_directories,
    }
}
