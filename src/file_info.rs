use rusqlite::{
    types::{FromSql, FromSqlError, FromSqlResult, Value, ValueRef},
    ToSql,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::SystemTime,
};
use xxhash_rust::xxh3::xxh3_128;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub hash: Option<String>,
    pub last_modified: SystemTime,
    pub last_accessed: SystemTime,
    pub tidy_score: Option<TidyScore>,
}

impl Default for FileInfo {
    fn default() -> Self {
        FileInfo {
            name: String::new(),
            path: PathBuf::new(),
            size: 0,
            hash: None,
            last_modified: SystemTime::UNIX_EPOCH,
            last_accessed: SystemTime::UNIX_EPOCH,
            tidy_score: None,
        }
    }
}

impl PartialEq for FileInfo {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TidyScore {
    pub misnamed: bool,
    pub unused: bool,
    pub duplicated: Option<Vec<FileInfo>>, // Not yet implemented
                                           // pub misplaced: bool,
}

impl TidyScore {
    pub const fn new(misnamed: bool, unused: bool, duplicated: Option<Vec<FileInfo>>) -> Self {
        Self {
            misnamed,
            unused,
            duplicated,
        }
    }
}

impl ToSql for TidyScore {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(rusqlite::types::ToSqlOutput::Owned(Value::from(
            serde_json::to_string(self).unwrap(),
        )))
    }
}

impl FromSql for TidyScore {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Text(s) => {
                let tidy_score: TidyScore =
                    match serde_json::from_str(match core::str::from_utf8(s) {
                        Ok(s) => s,
                        Err(_) => return Err(FromSqlError::InvalidType),
                    }) {
                        Ok(tidy_score) => tidy_score,
                        Err(_) => return Err(FromSqlError::InvalidType),
                    };
                Ok(tidy_score)
            }
            _ => Err(FromSqlError::InvalidType),
        }
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
                name: Path::new(path.to_str()?).file_name()?.to_str()?.to_owned(),
                path: path.clone(),
                size,
                hash: Some(file_signature.to_string()),
                last_modified,
                last_accessed,
                ..Default::default()
            })
        }
        Err(err) => {
            warn!("Could not get access to {} metadata: {}", path, err);
            None
        },
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
        let path = PathBuf::from(vec![r"tests", r"assets", r"test_folder", r"test-file-1"].iter().collect());
        let hash = get_file_signature(&path);
        assert_eq!(hash, 53180848542178601830765469314885156230);
    }

    #[test]
    fn test_create_file_info() {
        let path = PathBuf::from(vec![r"tests", r"assets", r"test_folder", r"test-file-1"].iter().collect());
        if let Some(file_info) = create_file_info(&path) {
            assert_eq!(file_info.path, path);
            assert_eq!(file_info.size, 100);
            if let Some(hash) = file_info.hash {
                assert_eq!(hash, "53180848542178601830765469314885156230");
            }
            assert_ne!(file_info.last_modified, SystemTime::UNIX_EPOCH);
            assert_ne!(file_info.last_accessed, SystemTime::UNIX_EPOCH);
        }
    }
}
