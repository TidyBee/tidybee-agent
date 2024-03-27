use crate::error::MyError;
use crate::file_info::{create_file_info, FileInfo};
use std::fs::read_dir;
use std::fs::DirEntry;
use std::path::PathBuf;

pub fn list_directories(directory: PathBuf) -> Result<Vec<FileInfo>, MyError> {
    let mut file_info_vec: Vec<FileInfo> = Vec::new();

    if directory.is_dir() {
        for dir_entry in read_dir(&directory)? {
            let dir_entry: DirEntry = dir_entry?;
            let dir_path: PathBuf = dir_entry.path();

            if dir_path.is_dir() {
                file_info_vec.extend(list_directories(dir_path)?);
            } else if dir_path.to_str().is_some() {
                if let Some(file_info) = create_file_info(&dir_path) {
                    file_info_vec.push(file_info);
                }
            }
        }
    } else {
        return Err(MyError::NotDirectory());
    }

    Ok(file_info_vec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid() {
        let res = list_directories(PathBuf::from("tests/assets/test_folder"));
        if let Ok(file_infos) = res {
            assert!(file_infos.iter().any(|file_info| file_info.pretty_path
                == PathBuf::from("tests/assets/test_folder/test-file-1")));
            assert!(file_infos.iter().any(|file_info| file_info.pretty_path
                == PathBuf::from("tests/assets/test_folder/test-file-1-dup-1")));
            assert!(file_infos.iter().any(|file_info| file_info.pretty_path
                == PathBuf::from("tests/assets/test_folder/test-file-1-dup-2")));
            assert!(file_infos.iter().any(|file_info| file_info.pretty_path
                == PathBuf::from("tests/assets/test_folder/test-file-1-dup-3")));
            assert!(file_infos.iter().any(|file_info| file_info.pretty_path
                == PathBuf::from("tests/assets/test_folder/test-file-2")));
            assert!(file_infos.iter().any(|file_info| file_info.pretty_path
                == PathBuf::from("tests/assets/test_folder/test-file-3")));
            assert!(file_infos.iter().any(|file_info| file_info.pretty_path
                == PathBuf::from("tests/assets/test_folder/test-file-4")));
            assert!(file_infos.iter().any(|file_info| file_info.pretty_path
                == PathBuf::from("tests/assets/test_folder/test-file-5")));
            assert!(file_infos.iter().any(|file_info| file_info.pretty_path
                == PathBuf::from("tests/assets/test_folder/test-file-6")));
            assert!(file_infos.iter().any(|file_info| file_info.pretty_path
                == PathBuf::from("tests/assets/test_folder/test-file-7")));
            assert!(file_infos.iter().any(|file_info| file_info.pretty_path
                == PathBuf::from("tests/assets/test_folder/test-file-8")));
            assert!(file_infos.iter().any(|file_info| file_info.pretty_path
                == PathBuf::from("tests/assets/test_folder/test-file-9")));
            assert!(file_infos.iter().any(|file_info| file_info.pretty_path
                == PathBuf::from("tests/assets/test_folder/test-file-10")));
            assert!(!file_infos
                .iter()
                .any(|file_info| file_info.pretty_path == PathBuf::from("file-does-not-exist")));
        }
    }

    #[test]
    fn empty_path() {
        let res = list_directories(PathBuf::from(""));
        assert!(res.is_err());
    }

    #[test]
    fn file_does_not_exist() {
        let res = list_directories(PathBuf::from("file-does-not-exist"));
        assert!(res.is_err());
    }

    #[test]
    fn is_reg_file() {
        let res = list_directories(PathBuf::from("tests/assets/test_folder/test-file-1"));
        assert!(res.is_err());
    }
}
