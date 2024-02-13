use crate::file_info::{create_file_info, fix_canonicalize_path, FileInfo};
use std::fs;
use std::path::PathBuf;

pub fn list_directories(directories: Vec<PathBuf>) -> Result<Vec<FileInfo>, std::io::Error> {
    let mut files: Vec<FileInfo> = Vec::new();

    for directory in directories {
        if directory.is_dir() {
            for entry in fs::read_dir(&directory)? {
                let entry: fs::DirEntry = entry?;
                let path: PathBuf = fix_canonicalize_path(fs::canonicalize(entry.path())?);

                if path.is_dir() {
                    files.extend(list_directories(vec![path])?);
                } else if path.to_str().is_some() {
                    if let Some(file_info) = create_file_info(&path) {
                        files.push(file_info);
                    }
                }
            }
        }
    }

    Ok(files)
}
