use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::SystemTime,
};
use tracing::warn;
use xxhash_rust::xxh3::xxh3_128;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub pretty_path: PathBuf,
    pub path: PathBuf,
    pub size: u64,
    pub hash: Option<String>,
    pub last_modified: SystemTime,
    pub last_accessed: SystemTime,
}

impl Default for FileInfo {
    fn default() -> Self {
        FileInfo {
            pretty_path: PathBuf::new(),
            path: PathBuf::new(),
            size: 0,
            hash: None,
            last_modified: SystemTime::UNIX_EPOCH,
            last_accessed: SystemTime::UNIX_EPOCH,
        }
    }
}

impl PartialEq for FileInfo {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

#[cfg(not(target_os = "windows"))]
pub fn fix_canonicalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    path.as_ref().into()
}

#[cfg(target_os = "windows")]
pub fn fix_canonicalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    const UNCPREFIX: &str = r"\\?\";
    let p: String = path.as_ref().display().to_string();
    if p.starts_with(UNCPREFIX) {
        p[UNCPREFIX.len()..].into()
    } else {
        p.into()
    }
}

pub fn get_file_signature(path: &PathBuf) -> u128 {
    let mut file = fs::File::open(path).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    xxh3_128(&buffer)
}

pub fn create_file_info(path: &PathBuf) -> Option<FileInfo> {
    match fs::metadata(path) {
        Ok(md) => {
            let size: u64 = md.len();
            let last_modified: SystemTime = md.modified().ok()?;
            let last_accessed: SystemTime = md.accessed().ok()?;
            let file_signature = get_file_signature(path);

            Some(FileInfo {
                pretty_path: fix_canonicalize_path(fs::canonicalize(path).unwrap()),
                path: fix_canonicalize_path(fs::canonicalize(path).unwrap()),
                size,
                hash: Some(file_signature.to_string()),
                last_modified,
                last_accessed,
                ..Default::default()
            })
        }
        Err(err) => {
            warn!("Could not get access to {:?} metadata: {}", path, err);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_fix_canonicalize_path_unix() {
        let path = "tests/assets/test_folder";
        let canonicalized = fix_canonicalize_path(path);
        assert_eq!(canonicalized, PathBuf::from(path));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_fix_canonicalize_path_windows() {
        let path = r"C:\tests\assets\test_folder";
        let canonicalized = fix_canonicalize_path(path);
        assert_eq!(canonicalized, PathBuf::from(path));
    }

    #[test]
    fn test_get_file_signature() {
        let path: PathBuf = [r"tests", r"assets", r"test_folder", r"test-file-1"]
            .iter()
            .collect();
        let hash = get_file_signature(&path);
        assert_eq!(hash, 53180848542178601830765469314885156230);
    }

    #[test]
    fn test_create_file_info() {
        let path: PathBuf = [r"tests", r"assets", r"test_folder", r"test-file-1"]
            .iter()
            .collect();
        if let Some(file_info) = create_file_info(&path) {
            assert_eq!(file_info.pretty_path, path);
            assert_eq!(file_info.size, 100);
            if let Some(hash) = file_info.hash {
                assert_eq!(hash, "53180848542178601830765469314885156230");
            }
            assert_ne!(file_info.last_modified, SystemTime::UNIX_EPOCH);
            assert_ne!(file_info.last_accessed, SystemTime::UNIX_EPOCH);
        }
    }
}
