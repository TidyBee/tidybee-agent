use crate::file_info::FileInfo;
use std::fs;
use std::path::{Path, PathBuf};

pub fn list_directories(directories: Vec<PathBuf>) -> Result<Vec<FileInfo>, std::io::Error> {
    let mut files: Vec<FileInfo> = Vec::new();

    for directory in directories {
        if directory.is_dir() {
            for entry in fs::read_dir(&directory)? {
                let entry: fs::DirEntry = entry?;
                let path: PathBuf = entry.path();

                if path.is_dir() {
                    files.extend(list_directories(vec![path])?);
                } else if let Some(file) = path.to_str() {
                    let md: fs::Metadata = fs::metadata(&path)?;
                    let size: u64 = md.len();
                    let last_modified: std::time::SystemTime = md.accessed()?;
                    files.push(FileInfo {
                        name: Path::new(file)
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned(),
                        path,
                        size,
                        last_modified,
                        ..Default::default()
                    });
                }
            }
        }
    }

    Ok(files)
}