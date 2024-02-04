use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, Value, ValueRef};
use rusqlite::ToSql;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

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
