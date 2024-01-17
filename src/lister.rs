use crate::file_info::FileInfo;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use xxhash_rust::xxh3::xxh3_128;

fn get_file_signature(path: &PathBuf) -> u128 {
    let mut file = fs::File::open(path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    xxh3_128(&buffer)
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
                } else if let Some(file) = path.to_str() {
                    let md: fs::Metadata = fs::metadata(&path)?;
                    let size: u64 = md.len();
                    let last_modified: std::time::SystemTime = md.accessed()?;
                    let file_signature = get_file_signature(&path);
                    files.push(FileInfo {
                        name: Path::new(file)
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned(),
                        path,
                        size,
                        hash: Some(file_signature.to_string()),
                        last_modified,
                        ..Default::default()
                    });
                }
            }
        }
    }

    Ok(files)
}
