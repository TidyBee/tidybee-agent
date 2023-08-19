use std::path::PathBuf;

pub struct Config {
    pub file_extensions: Option<Vec<String>>,
    pub file_types: Option<Vec<String>>,
    pub list_directories: Option<Vec<PathBuf>>,
    pub watch_directories: Option<Vec<PathBuf>>,
}

pub enum ParseError {
    ConflictingArguments(String),
}

pub fn get_options() -> Result<Config, ParseError> {
    let matches = clap::App::new("TidyBee")
        .version("0.0.1")
        .author("majent4")
        .about("Watch for file changes and list directories")
        .arg(
            clap::Arg::with_name("extension")
                .short("e")
                .long("extension")
                .value_name("EXTENSIONS")
                .multiple(true)
                .use_delimiter(true)
                .takes_value(true)
                .help("Specify extensions"),
        )
        .arg(
            clap::Arg::with_name("type")
                .short("t")
                .long("type")
                .value_name("TYPES")
                .multiple(true)
                .use_delimiter(true)
                .takes_value(true)
                .help("Specify types"),
        )
        .arg(
            clap::Arg::with_name("list")
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
            clap::Arg::with_name("watch")
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

    let file_extensions: Option<Vec<String>> = matches
        .values_of("extension")
        .map(|exts: clap::Values<'_>| exts.map(String::from).collect());
    let file_types: Option<Vec<String>> = matches
        .values_of("type")
        .map(|file_types: clap::Values<'_>| file_types.map(String::from).collect());
    let list_directories: Option<Vec<PathBuf>> = matches
        .values_of("list")
        .map(|dirs: clap::Values<'_>| dirs.map(PathBuf::from).collect());
    let watch_directories: Option<Vec<PathBuf>> = matches
        .values_of("watch")
        .map(|dirs: clap::Values<'_>| dirs.map(PathBuf::from).collect());
    if list_directories.is_some() && watch_directories.is_some() {
        return Err(ParseError::ConflictingArguments(
            "-l and -w cannot be provided at the same time.".to_string(),
        ));
    }

    Ok(Config {
        file_extensions,
        file_types,
        list_directories,
        watch_directories,
    })
}
