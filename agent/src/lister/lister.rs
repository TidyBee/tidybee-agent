use serde::{Deserialize, Serialize};
use std::fs;
use std::path;

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub name: String,
    pub size: u64,
}

pub fn list_directories(directories: Vec<path::PathBuf>) -> Result<Vec<File>, std::io::Error> {
    let mut files: Vec<File> = Vec::new();

    for d in directories {
        if d.is_dir() {
            for entry in fs::read_dir(&d)? {
                let entry: fs::DirEntry = entry?;
                let path: path::PathBuf = entry.path();

                if path.is_dir() {
                    files.extend(list_directories(vec![path])?);
                } else {
                    if let Some(file) = path.to_str() {
                        let md: fs::Metadata = fs::metadata(&path)?;
                        let size: u64 = md.len();
                        files.push(File {
                            name: file.to_string(),
                            size,
                        });
                    }
                }
            }
        }
    }

    Ok(files)
}
