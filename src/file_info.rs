use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, Value, ValueRef};
use rusqlite::ToSql;
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
    pub unused: bool,
    pub duplicated: Vec<FileInfo>,

    // Not yet implemented
    // pub misplaced: bool,
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
                    match serde_json::from_str(match std::str::from_utf8(s) {
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
