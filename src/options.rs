use std::path;

pub struct Options {
    pub file_extensions: Option<Vec<String>>,
    pub file_types: Option<String>,
    pub list_directories: Option<Vec<path::PathBuf>>,
    pub watch_directories: Option<Vec<path::PathBuf>>,
    pub receive_address: Option<String>,
    pub send_address: Option<String>,
}

pub enum OptionsError {
    ConflictingOptions(String),
    InvalidDirectory(String),
    InvalidFileType(String),
    InvalidFileExtension(String),
}

pub fn get_options() -> Result<Options, OptionsError> {
    check_options(clap_options().get_matches())
}

fn clap_options() -> clap::App<'static, 'static> {
    let options: clap::App<'_, '_> = clap::App::new("TidyBee Watch & List")
        .version("0.0.1")
        .author("majent4")
        .about("Watch for changes in directories and recursively list directories")
        .arg(
            clap::Arg::with_name("extension")
                .short("e")
                .long("extension")
                .value_name("EXTENSIONS")
                .multiple(true)
                .use_delimiter(true)
                .takes_value(true)
                .help("Specify file extensions"),
        )
        .arg(
            clap::Arg::with_name("type")
                .short("t")
                .long("type")
                .value_name("TYPES")
                .multiple(true)
                .use_delimiter(true)
                .takes_value(true)
                .help("Specify file types"),
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
        .arg(
            clap::Arg::with_name("receive")
                .short("r")
                .long("receive")
                .value_name("ADDRESS")
                .takes_value(true)
                .help("Specify receive address"),
        )
        .arg(
            clap::Arg::with_name("send")
                .short("s")
                .long("send")
                .value_name("ADDRESS")
                .takes_value(true)
                .help("Specify send address"),
        );
    options
}

fn check_options(matches: clap::ArgMatches<'_>) -> Result<Options, OptionsError> {
    let list_directories: Option<Vec<path::PathBuf>> = matches
        .values_of("list")
        .map(|dirs: clap::Values<'_>| dirs.map(path::PathBuf::from).collect());

    let watch_directories: Option<Vec<path::PathBuf>> = matches
        .values_of("watch")
        .map(|dirs: clap::Values<'_>| dirs.map(path::PathBuf::from).collect());

    if list_directories.is_some() && watch_directories.is_some() {
        return Err(OptionsError::ConflictingOptions(
            "can't specify both list and watch simultaneously".to_string(),
        ));
    }

    if let Some(directories) = &list_directories {
        for d in directories {
            if !d.is_dir() {
                return Err(OptionsError::InvalidDirectory(format!(
                    "specified directory does not exists: {:?}",
                    d
                )));
            }
        }
    }

    if let Some(directories) = &watch_directories {
        for d in directories {
            if !d.is_dir() {
                return Err(OptionsError::InvalidDirectory(format!(
                    "specified directory does not exists: {:?}",
                    d
                )));
            }
        }
    }

    let file_extensions: Option<Vec<String>> = matches
        .values_of("extension")
        .map(|exts: clap::Values<'_>| exts.map(String::from).collect());

    let valid_extensions: Vec<&str> =
        vec!["docx", "jpeg", "jpg", "mp3", "mp4", "pdf", "png", "xlsx"];

    if let Some(file_extensions) = &file_extensions {
        for e in file_extensions {
            if !valid_extensions.contains(&e.as_str()) {
                return Err(OptionsError::InvalidFileExtension(format!(
                    "invalid file extension: {}",
                    e
                )));
            }
        }
    }

    let file_types_vec: Option<Vec<String>> = matches
        .values_of("type")
        .map(|file_types: clap::Values<'_>| file_types.map(String::from).collect());

    let valid_file_types: Vec<&str> = vec!["*", "all", "directory", "regular"];

    if let Some(file_types) = &file_types_vec {
        for t in file_types {
            if !valid_file_types.contains(&t.as_str()) {
                return Err(OptionsError::InvalidFileType(format!(
                    "invalid file type: {}",
                    t
                )));
            }
        }
    }

    let mut file_types: Option<String> = None;

    if let Some(t) = &file_types_vec {
        if t.iter().any(|t: &String| t.contains("directory")) {
            file_types = Some("directory".to_string());
        }
        if t.iter()
            .any(|t: &String| t.contains("*") || t.contains("all"))
        {
            file_types = Some("all".to_string());
        }
        if t.iter().any(|t: &String| t.contains("regular")) {
            file_types = Some("regular".to_string());
        }
    }

    if let Some(file_extensions) = &file_extensions {
        if let Some(file_types) = &file_types {
            if file_types == "directory" && !file_extensions.is_empty() {
                return Err(OptionsError::InvalidFileType(
                    "can't specify both file extensions and file type directory simultaneously"
                        .to_string(),
                ));
            }
        }
    }

    let receive_address: Option<String> = matches.value_of("receive").map(String::from);

    let send_address: Option<String> = matches.value_of("send").map(String::from);

    Ok(Options {
        file_extensions,
        file_types,
        list_directories,
        watch_directories,
        receive_address,
        send_address,
    })
}

pub fn print_option_error(error: OptionsError) {
    match error {
        OptionsError::ConflictingOptions(e) => {
            eprintln!("tidybee: error: {}", e);
        }
        OptionsError::InvalidDirectory(e) => {
            eprintln!("tidybee: error: {}", e);
        }
        OptionsError::InvalidFileType(e) => {
            eprintln!("tidybee: error: {}", e);
        }
        OptionsError::InvalidFileExtension(e) => {
            eprintln!("tidybee: error: {}", e);
        }
    }
}

// to do: how to get stderr?
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easy_list_valid() {
        let arguments: Vec<&str> = vec!["tidybee", "--list", "/usr"];
        let options: clap::ArgMatches<'_> = clap_options().get_matches_from(arguments);
        assert!(check_options(options).is_ok());
    }

    #[test]
    fn test_easy_watch_valid() {
        let arguments: Vec<&str> = vec!["tidybee", "--watch", "/usr"];
        let options: clap::ArgMatches<'_> = clap_options().get_matches_from(arguments);
        assert!(check_options(options).is_ok());
    }

    #[test]
    fn test_easy_help() {
        let arguments: Vec<&str> = vec!["tidybee", "--help"];
        let options: clap::ArgMatches<'_> = clap_options().get_matches_from(arguments);
        assert!(check_options(options).is_ok());
    }

    #[test]
    fn test_easy_version() {
        let arguments: Vec<&str> = vec!["tidybee", "--version"];
        let options: clap::ArgMatches<'_> = clap_options().get_matches_from(arguments);
        assert!(check_options(options).is_ok());
    }

//    #[test]
//    fn test_easy_empty() {
//        let arguments: Vec<&str> = vec!["tidybee"];
//        let options: clap::ArgMatches<'_> = clap_options().get_matches_from(arguments);
//        assert!(check_options(options).is_err());
//    }

    #[test]
    fn test_directory_extension() {
        let arguments: Vec<&str> = vec!["tidybee", "--list", "/usr", "-e", "pdf", "-t", "directory" ];
        let options: clap::ArgMatches<'_> = clap_options().get_matches_from(arguments);
        assert!(check_options(options).is_err());
    }
}

