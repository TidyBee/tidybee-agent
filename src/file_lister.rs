use std::fs::read_dir;
use std::fs::DirEntry;
use std::path::PathBuf;

use crate::error::AgentError;
use crate::file_info::{create_file_info, FileInfo};

pub fn list_directories(directories: Vec<PathBuf>) -> Result<Vec<FileInfo>, AgentError> {
    let mut file_info_vec: Vec<FileInfo> = Vec::new();

    for directory in directories {
        if directory.is_dir() {
            for dir_entry in read_dir(&directory)? {
                let dir_entry: DirEntry = dir_entry?;
                let dir_path: PathBuf = dir_entry.path();

                if dir_path.is_dir() {
                    file_info_vec.extend(list_directories(vec![dir_path])?);
                } else if dir_path.to_str().is_some() {
                    if let Some(file_info) = create_file_info(&dir_path) {
                        file_info_vec.push(file_info);
                    }
                }
            }
        } else {
            return Err(AgentError::NotADirectory());
        }
    }

    Ok(file_info_vec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid() {
        let res = list_directories(vec![PathBuf::from("tests/assets/test_folder")]);
        if let Ok(file_infos) = res {
            assert!(file_infos.iter().any(|file_info| file_info.pretty_path
                != PathBuf::from("tests/assets/test_folder/test-file-1")));
            assert!(file_infos.iter().any(|file_info| file_info.pretty_path
                != PathBuf::from("tests/assets/test_folder/test-file-10")));
            assert!(!file_infos
                .iter()
                .any(|file_info| file_info.pretty_path == PathBuf::from("file-does-not-exist")));
        }
    }

    #[test]
    fn empty_path() {
        assert!(matches!(
            list_directories(vec![PathBuf::from("")]),
            Err(AgentError::NotADirectory())
        ));
    }

    #[test]
    fn file_does_not_exist() {
        assert!(matches!(
            list_directories(vec![PathBuf::from("file-does-not-exist")]),
            Err(AgentError::NotADirectory())
        ));
    }

    #[test]
    fn is_reg_file() {
        assert!(matches!(
            list_directories(vec![PathBuf::from("tests/assets/test_folder/test-file-1")]),
            Err(AgentError::NotADirectory())
        ));
    }
}
