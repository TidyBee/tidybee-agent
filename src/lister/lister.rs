use crate::file_info::FileInfo;
use log::warn;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use xxhash_rust::xxh3::xxh3_64 as hasher;

fn get_hash_from_file(file: &PathBuf) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(file)?;
    let mut buffer = Vec::new();
    // Future optimization: use multithreading to read the file and compute the hash
    file.read_to_end(&mut buffer)?;
    let hash = hasher(&buffer);
    Ok(format!("{:x}", hash))
}

pub fn list_directories(directories: Vec<PathBuf>) -> Result<Vec<FileInfo>, std::io::Error> {
    let mut files: Vec<FileInfo> = Vec::new();

    for directory in directories {
        if directory.is_dir() {
            for entry in fs::read_dir(&directory)? {
                let entry: fs::DirEntry = entry?;
                let path: PathBuf = entry.path();

                if path.is_dir() {
                    files.extend(list_directories(vec![path])?);
                } else {
                    if let Some(file) = path.to_str() {
                        let md: fs::Metadata = fs::metadata(&path)?;
                        let size: u64 = md.len();
                        let last_modified: std::time::SystemTime = md.accessed()?;
                        let file_digest = match get_hash_from_file(&path) {
                            Ok(digest) => Some(digest),
                            Err(_) => {
                                warn!("Could not get hash from file: {:?}", file);
                                None
                            }
                        };
                        files.push(FileInfo {
                            name: Path::new(file)
                                .file_name()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string(),
                            path,
                            size,
                            last_modified,
                            hash: file_digest,
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    Ok(files)
}
