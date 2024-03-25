use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, error};

use crate::{file_info, my_files, tidy_algo::TidyAlgo};

/// Converts a given path to a pretty path by removing the common root paths.
///
/// # Arguments
///
/// * `path` - A reference to the `PathBuf` representing the original path.
/// * `roots` - A vector of `PathBuf` representing the root paths.
///
/// # Returns
///
/// A `PathBuf` representing the pretty path.
pub fn path_to_pretty_path(path: &PathBuf, roots: Vec<PathBuf>) -> PathBuf {
    let path_str = path.to_str().unwrap();
    let mut pretty_path = path_str.to_string();
    for root in roots {
        let root_str = root.to_str().unwrap();
        if let Some(root_str_index) = path_str.find(root_str) {
            pretty_path = path_str[root_str_index..].to_string();
            break;
        }
    }

    PathBuf::from(pretty_path)
}

/// Updates the file information, tidy score, and grade for a given file.
///
/// # Arguments
///
/// * `path` - A reference to the `PathBuf` representing the file path.
/// * `my_files` - A reference to the `MyFiles` struct.
/// * `tidy_algo` - A reference to the `TidyAlgo` struct.
pub fn update_file(path: &PathBuf, my_files: &my_files::MyFiles, tidy_algo: &TidyAlgo) {
    if let Some(mut file_info) = file_info::create_file_info(&path.clone()) {
        let _ = my_files.update_fileinfo(file_info.clone());
        debug!("log: {:?}", file_info);
        tidy_algo.apply_rules(&mut file_info, my_files);
        if let Some(tidyscore) = file_info.tidy_score {
            let _ = my_files.set_tidyscore(path.clone(), &tidyscore);
        }
        my_files.update_grade(path.clone(), tidy_algo);
    }
}

/// Updates the grade for all files in the database.
///
/// # Arguments
///
/// * `my_files` - A reference to the `MyFiles` struct.
/// * `tidy_algo` - A reference to the `TidyAlgo` struct.
pub fn update_all_grades(my_files: &my_files::MyFiles, tidy_algo: &TidyAlgo) {
    let files = my_files.get_all_files_from_db();
    match files {
        Ok(files) => {
            for file in files {
                let file_path = file.path.clone();
                my_files.update_grade(file_path, tidy_algo);
            }
        }
        Err(error) => {
            error!("{:?}", error);
        }
    }
}

// region: Safe file operations

/// Adds a file to the database in a safe manner.
///
/// # Arguments
///
/// * `path` - A `PathBuf` representing the file path.
/// * `my_files` - A reference to the `MyFiles` struct.
pub async fn safe_add_file_to_db(path: &PathBuf, my_files: &my_files::MyFiles) {
    if fs::metadata(path.clone()).await.is_ok() {
        if let Some(file) = file_info::create_file_info(&path.clone()) {
            match my_files.add_file_to_db(&file) {
                Ok(_) => {}
                Err(error) => {
                    error!("{error:?}");
                }
            }
        }
    } else {
        error!(
            "Trying to add in the database a file that does not exist: {}",
            path.display()
        );
    }
}

/// Removes a file from the database in a safe manner.
///
/// # Arguments
///
/// * `path` - A `PathBuf` representing the file path.
/// * `my_files` - A reference to the `MyFiles` struct.
pub async fn safe_remove_file_from_db(path: PathBuf, my_files: &my_files::MyFiles) {
    if fs::metadata(path.clone()).await.is_err() {
        match my_files.remove_file_from_db(path.clone()) {
            Ok(_) => {}
            Err(error) => {
                error!("{error:?}");
            }
        }
    } else {
        error!(
            "Trying to remove from the database a file that exists: {}",
            path.display()
        );
    }
}

// endregion: Safe file operations