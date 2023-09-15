use crate::tidyalgo::FileInfo;
use std::fs;
use std::path;

pub fn list_directories(directories: Vec<path::PathBuf>) -> Result<Vec<FileInfo>, std::io::Error> {
    let mut files: Vec<FileInfo> = Vec::new();

    for directory in directories {
        if directory.is_dir() {
            for entry in fs::read_dir(&directory)? {
                let entry: fs::DirEntry = entry?;
                let path: path::PathBuf = entry.path();

                if path.is_dir() {
                    files.extend(list_directories(vec![path])?);
                } else {
                    if let Some(file) = path.to_str() {
                        let md: fs::Metadata = fs::metadata(&path)?;
                        let size: u64 = md.len();
                        files.push(FileInfo {
                            name: file.to_string(),
                            size,
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    Ok(files)
}
