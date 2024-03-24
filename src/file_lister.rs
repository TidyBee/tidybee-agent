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
    }

    Ok(file_info_vec)
}
