use std::path;

pub struct Options {
    pub file_extensions: Option<Vec<String>>,
    pub file_types: Option<Vec<String>>,
    pub list_directories: Option<Vec<path::PathBuf>>,
    pub watch_directories: Option<Vec<path::PathBuf>>,
}

pub enum OptionsError {
    ConflictingOptions(String),
    InvalidDirectory(String),
    InvalidFileType(String),
    InvalidFileExtension(String)
}

pub fn get_options() -> Result<Options, OptionsError> {
    let options: clap::ArgMatches<'_> = clap::App::new("TidyBee Core")
        .version("0.0.1")
        .author("majent4")
        .about("Watch for changes in directories, recursively list directories")
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

    let list_directories: Option<Vec<path::PathBuf>> = options
        .values_of("list")
        .map(|dirs: clap::Values<'_>| dirs.map(path::PathBuf::from).collect());

    let watch_directories: Option<Vec<path::PathBuf>> = options
        .values_of("watch")
        .map(|dirs: clap::Values<'_>| dirs.map(path::PathBuf::from).collect());

    if list_directories.is_some() && watch_directories.is_some() {
        return Err(OptionsError::ConflictingOptions(
            "Can't specify both list and watch".to_string(),
        ));
    }

    if let Some(directories) = &list_directories {
        for d in directories {
            if !d.is_dir() {
                return Err(OptionsError::InvalidDirectory(format!(
                    "Specified directory {:?} does not exists",
                    d
                )));
            }
        }
    }

    if let Some(directories) = &watch_directories {
        for d in directories {
            if !d.is_dir() {
                return Err(OptionsError::InvalidDirectory(format!(
                    "Specified directory {:?} does not exists",
                    d
                )));
            }
        }
    }

    let file_extensions: Option<Vec<String>> = options
        .values_of("extension")
        .map(|exts: clap::Values<'_>| exts.map(String::from).collect());

    let valid_extensions = vec!["txt", "docx", "pdf", "ai", "xlsx"];

    if let Some(file_extensions) = &file_extensions {
        for e in file_extensions {
            if !valid_extensions.contains(&e.as_str()) {
                return Err(OptionsError::InvalidFileExtension(format!(
                    "Invalid file extension: {}",
                    e
                )));
            }
        }
    }

    let file_types: Option<Vec<String>> = options
        .values_of("type")
        .map(|file_types: clap::Values<'_>| file_types.map(String::from).collect());

    let valid_file_types: Vec<&str> = vec!["regular", "all", "directories"];

    if let Some(file_types) = &file_types {
        for ft in file_types {
            if !valid_file_types.contains(&ft.as_str()) {
                return Err(OptionsError::InvalidFileType(format!(
                    "Invalid file type: {}",
                    ft
                )));
            }
        }
    }

    Ok(Options {
        file_extensions,
        file_types,
        list_directories,
        watch_directories,
    })
}
