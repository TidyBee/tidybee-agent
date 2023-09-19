use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub name: String,
    pub path: std::path::PathBuf,
    pub size: u64,
    pub last_modified: std::time::SystemTime,
    pub tidy_score: Option<TidyScore>,
}

impl Default for FileInfo {
    fn default() -> Self {
        FileInfo {
            name: "".to_string(),
            path: std::path::PathBuf::new(),
            size: 0,
            last_modified: std::time::SystemTime::UNIX_EPOCH,
            tidy_score: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TidyScore {
    pub misnamed: bool,
    pub misplaced: bool,
    pub unused: bool,
    pub duplicated: Vec<FileInfo>,
}
