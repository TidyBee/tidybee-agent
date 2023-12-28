use crate::configuration_wrapper::ConfigurationWrapper;
use crate::file_info::{FileInfo, TidyScore};
use chrono::{DateTime, Utc};
use itertools::{Either, Itertools};
use log::{error, info, warn};
use r2d2;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Result, ToSql};
use serde::{Deserialize, Serialize};
use std::path;
use std::path::PathBuf;

// region: --- MyFiles builder states
#[derive(Default, Clone)]
pub struct Sealed;

#[derive(Default, Clone)]
pub struct NotSealed;

#[derive(Default, Clone)]
pub struct NoConfigurationWrapper;

#[derive(Default, Clone)]
pub struct NoConnectionManager;

#[derive(Default, Clone)]
pub struct ConfigurationWrapperPresent(ConfigurationWrapper);

#[derive(Clone)]
pub struct ConnectionManagerPresent(Pool<SqliteConnectionManager>);
// endregion: --- MyFiles builder states

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MyFilesDatabaseConfiguration {
    pub db_path: String,
    pub drop_db_on_start: bool,
}

pub struct MyFiles {
    connection_pool: PooledConnection<SqliteConnectionManager>,
    configuration: MyFilesDatabaseConfiguration,
}

#[derive(Copy, Clone, Default)]
pub struct MyFilesBuilder<C, M, S> {
    connection_manager: M,
    configuration_wrapper_instance: C,
    marker_seal: std::marker::PhantomData<S>,
}

impl Default for ConnectionManagerPresent {
    fn default() -> Self {
        ConnectionManagerPresent(Pool::new(SqliteConnectionManager::file("my_files.db")).unwrap())
    }
}

impl MyFilesBuilder<NoConfigurationWrapper, NoConnectionManager, NotSealed> {
    pub fn new() -> Self {
        MyFilesBuilder {
            connection_manager: NoConnectionManager,
            configuration_wrapper_instance: NoConfigurationWrapper,
            marker_seal: std::marker::PhantomData,
        }
    }
}

impl<C, M> MyFilesBuilder<C, M, NotSealed> {
    pub fn configuration_wrapper(
        self,
        configuration_wrapper_instance: ConfigurationWrapper,
    ) -> MyFilesBuilder<ConfigurationWrapperPresent, ConnectionManagerPresent, NotSealed> {
        let db_path = configuration_wrapper_instance
            .bind::<MyFilesDatabaseConfiguration>("my_files_database_configuration")
            .unwrap_or_default()
            .db_path;
        let manager = SqliteConnectionManager::file(db_path);
        let pool = match Pool::new(manager) {
            Ok(pool) => pool,
            Err(error) => {
                error!("Error creating connection pool: {}", error);
                panic!();
            }
        };

        MyFilesBuilder {
            configuration_wrapper_instance: ConfigurationWrapperPresent(
                configuration_wrapper_instance,
            ),
            connection_manager: ConnectionManagerPresent(pool),
            marker_seal: std::marker::PhantomData,
        }
    }
    pub fn seal(self) -> MyFilesBuilder<C, M, Sealed> {
        MyFilesBuilder {
            connection_manager: self.connection_manager,
            configuration_wrapper_instance: self.configuration_wrapper_instance,
            marker_seal: std::marker::PhantomData,
        }
    }
}

impl MyFilesBuilder<ConfigurationWrapperPresent, ConnectionManagerPresent, Sealed> {
    pub fn build(&self) -> Result<MyFiles> {
        let my_files_configuration = self
            .configuration_wrapper_instance
            .0
            .bind::<MyFilesDatabaseConfiguration>("my_files_database_configuration")
            .unwrap_or_default();
        let connection_pool = match self.connection_manager.0.get() {
            Ok(connection) => connection,
            Err(error) => {
                error!("Error getting connection from pool: {}", error);
                panic!();
            }
        };
        MyFiles::new(my_files_configuration, connection_pool)
    }
}

#[allow(dead_code)]
impl MyFiles {
    pub fn new(
        configuration: MyFilesDatabaseConfiguration,
        connection_pool: PooledConnection<SqliteConnectionManager>,
    ) -> Result<Self> {
        Ok(MyFiles {
            connection_pool,
            configuration,
        })
    }
    pub fn init_db(&self) -> Result<(), rusqlite::Error> {
        if self.configuration.drop_db_on_start {
            let drop_db = match self.connection_pool.execute_batch(
                "
                BEGIN;
                DROP TABLE IF EXISTS duplicates_associative_table;
                DROP TABLE IF EXISTS tidy_scores;
                DROP TABLE IF EXISTS my_files;
                COMMIT;",
            ) {
                Ok(_) => Ok(info!("Database dropped")),
                Err(error) => {
                    error!("Error dropping database: {}", error);
                    Err(error)
                }
            };
            drop_db?;
        }
        match self.connection_pool.execute_batch(
            "
            BEGIN;
            CREATE TABLE IF NOT EXISTS my_files (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                name            TEXT NOT NULL,
                path            TEXT NOT NULL UNIQUE,
                size            INTEGER NOT NULL,
                hash            TEXT DEFAULT \"\",
                last_modified   DATE NOT NULL,
                tidy_score      INTEGER UNIQUE,
                FOREIGN KEY (tidy_score) REFERENCES tidy_scores(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS tidy_scores (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                misnamed        BOOLEAN NOT NULL,
                unused          BOOLEAN NOT NULL,
                duplicated      BOOLEAN NOT NULL
            );
            CREATE TABLE IF NOT EXISTS duplicates_associative_table (
                original_file_id    INTEGER NOT NULL,
                duplicated_file_id  INTEGER NOT NULL,
                PRIMARY KEY (original_file_id, duplicated_file_id),
                FOREIGN KEY (original_file_id) REFERENCES my_files(id) ON DELETE CASCADE,
                FOREIGN KEY (duplicated_file_id) REFERENCES my_files(id) ON DELETE CASCADE
            );
            COMMIT;",
        ) {
            Ok(_) => {
                info!("Database initialized");
                Ok(())
            }
            Err(error) => {
                error!("Error initializing database: {}", error);
                Err(error)
            }
        }
    }
    pub fn remove_file_from_db(&self, file_path: &str) -> Result<()> {
        match self
            .connection_pool
            .execute("DELETE FROM my_files WHERE path = ?1", params![file_path])
        {
            Ok(_) => {
                info!("{} removed from my_files", file_path);
                Ok(())
            }
            Err(error) => {
                error!("Error removing {} from my_files: {}", file_path, error);
                Err(error)
            }
        }
    }
    pub fn add_file_to_db(&self, file: &FileInfo) -> Result<FileInfo, ()> {
        let last_modified: DateTime<Utc> = file.last_modified.into();
        match self.connection_pool.execute(
            "INSERT INTO my_files (name, path, size, hash, last_modified, tidy_score)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                file.name,
                file.path.to_str(),
                file.size,
                file.hash,
                last_modified.to_rfc3339(),
                file.tidy_score.as_ref()
            ],
        ) {
            Ok(_) => Ok({
                info!("{} added to my_files", file.path.to_str().unwrap());
                file.clone()
            }),
            Err(error) => {
                warn!(
                    "Error adding {} to my_files: {}",
                    file.path.to_str().unwrap(),
                    error
                );
                Err(())
            }
        }
    }

    pub fn add_duplicated_file_to_db(
        &self,
        file_path: PathBuf,
        duplicated_file_path: PathBuf,
    ) -> Result<()> {
        let str_filepath = file_path.to_str().unwrap();

        let mut statement = self
            .connection_pool
            .prepare(
                "SELECT id, tidy_score FROM my_files
                WHERE path = ?1",
            )
            .unwrap();
        let result = statement
            .query_row(params![&str_filepath], |row| {
                Ok((row.get::<_, i64>(0), row.get::<_, i64>(1)))
            })
            .unwrap();
        let file_id = result.0?;
        let tidy_score_id = result.1?;

        let mut statement = self
            .connection_pool
            .prepare(
                "SELECT id FROM my_files
            WHERE path = ?1",
            )
            .unwrap();
        let duplicated_file_id = statement
            .query_row(
                params![duplicated_file_path.clone().into_os_string().to_str()],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();

        let mut statement = self
            .connection_pool
            .prepare(
                "INSERT INTO duplicates_associative_table (original_file_id, duplicated_file_id)
            VALUES (?1, ?2)",
            )
            .unwrap();
        let _ = match statement.execute(params![file_id, duplicated_file_id]) {
            Ok(_) => Ok(info!(
                "{:?} added to duplicates_associative_table",
                str_filepath
            )),
            Err(error) => {
                error!(
                    "Error adding {:?} with id {} to duplicates_associative_table: {}",
                    str_filepath, file_id, error
                );
                Err(error)
            }
        };
        // Set tidy_score to duplicated = true
        let mut statement = self
            .connection_pool
            .prepare(
                "
            UPDATE tidy_scores SET duplicated = true
            WHERE id = ?1",
            )
            .unwrap();
        match statement.execute(params![tidy_score_id]) {
            Ok(_) => Ok(info!(
                "{:?} added to duplicates_associative_table",
                str_filepath
            )),
            Err(error) => {
                error!(
                    "Error adding {:?} with id {} to duplicates_associative_table: {}",
                    file_path, file_id, error
                );
                Err(error)
            }
        }
    }

    fn create_fileinfo_from_row(row: &rusqlite::Row) -> Result<FileInfo> {
        let path_str = row.get::<_, String>(2)?;
        let path = std::path::Path::new(&path_str).to_owned();

        let time_str = row.get::<_, String>(5)?;
        if DateTime::parse_from_rfc3339(&time_str).is_err() {
            error!(
                "MyFiles::create_fileinfo_from_row: Error parsing key: last_modified with value {}, for file {}.",
                time_str, path_str
            );
        }
        let last_modified = DateTime::parse_from_rfc3339(&time_str).unwrap().into();

        let tidy_score = match row.get::<_, Option<TidyScore>>(6) {
            Ok(tidy_score) => tidy_score,
            Err(error) => {
                error!(
                    "MyFiles::create_fileinfo_from_row: Error parsing key: tidy_score for file {} : {}",
                    path_str, error
                );
                None
            }
        };

        Ok(FileInfo {
            name: row.get::<_, String>(1)?,
            path,
            size: row.get::<_, u64>(3)?,
            hash: row.get::<_, Option<String>>(4)?,
            last_modified,
            tidy_score,
        })
    }

    pub fn get_all_files_from_db(&self) -> Result<Vec<FileInfo>> {
        let mut statement = self.connection_pool.prepare("SELECT * FROM my_files")?;
        let file_iter_res = statement.query_map(params![], MyFiles::create_fileinfo_from_row);

        let file_iter = match file_iter_res {
            Ok(file_iter) => file_iter,
            Err(error) => {
                error!(
                    "MyFiles::get_all_files_from_db: Error getting file iterator from database: {}",
                    error
                );
                return Err(error);
            }
        };
        let (files_vec, errs): (Vec<FileInfo>, Vec<rusqlite::Error>) =
            file_iter.partition_map(|file| match file {
                Ok(file) => Either::Left(file),
                Err(error) => Either::Right(error),
            });
        for err in errs {
            error!(
                "MyFiles::get_all_files_from_db: Error getting file from database: {:?}",
                err
            );
        }
        Ok(files_vec)
    }

    pub fn fetch_duplicated_files(&self, file_path: PathBuf) -> Result<Vec<FileInfo>> {
        let str_file_path = file_path.to_str().unwrap();

        // Check if file is duplicated
        let mut duplicate_check_stmt = self.connection_pool.prepare(
            "SELECT duplicated FROM tidy_scores INNER JOIN my_files ON tidy_scores.id = my_files.tidy_score WHERE my_files.path = ?1",
        )?;
        let duplicated =
            duplicate_check_stmt.query_row(params![&str_file_path], |row| row.get::<_, bool>(0))?;

        if !duplicated {
            info!("No duplicate found while checking {:?}", file_path);
            return Ok(Vec::new());
        }

        let mut statement = self
            .connection_pool
            .prepare("SELECT id FROM my_files WHERE path = ?1")?;
        let file_id = statement.query_row(params![&str_file_path], |row| row.get::<_, i64>(0))?;

        let mut statement = self.connection_pool.prepare(
            "SELECT my_files.name, my_files.path, my_files.size, my_files.last_modified, my_files.hash, my_files.tidy_score
            FROM my_files
            INNER JOIN duplicates_associative_table ON my_files.id = duplicates_associative_table.original_file_id WHERE duplicates_associative_table.original_file_id = ?1",
        ).unwrap();

        let duplicated_files = statement
            .query_map(params![file_id], |row| {
                let path_str = row.get::<_, String>(0)?;
                let path = std::path::Path::new(&path_str).to_owned();

                let time_str = row.get::<_, String>(3)?;
                let last_modified = match DateTime::parse_from_rfc3339(&time_str) {
                    Ok(last_modified) => last_modified.into(),
                    Err(error) => {
                        error!(
                            "Error parsing key: last_modified with value {}, for file {}. {}",
                            time_str, path_str, error
                        );
                        std::time::SystemTime::UNIX_EPOCH
                    }
                };

                let tidy_score_id = row.get::<_, i64>(5)?;
                let mut statement = self.connection_pool.prepare(
                    "SELECT misnamed, duplicated, unused FROM tidy_scores WHERE id = ?1",
                )?;
                let tidy_score = statement.query_row(params![tidy_score_id], |row| {
                    Ok(TidyScore {
                        misnamed: row.get::<_, bool>(0)?,
                        duplicated: None,
                        unused: row.get::<_, bool>(2)?,
                    })
                })?;

                Ok(FileInfo {
                    name: row.get::<_, String>(0)?,
                    path,
                    size: row.get::<_, u64>(2)?,
                    hash: row.get::<_, Option<String>>(4)?,
                    last_modified,
                    tidy_score: tidy_score.into(),
                })
            })
            .unwrap();
        let mut duplicated_files_vec: Vec<FileInfo> = Vec::new();
        for file in duplicated_files {
            duplicated_files_vec.push(file.unwrap());
        }
        Ok(duplicated_files_vec)
    }

    pub fn get_tidyscore(&self, file_path: PathBuf) -> Result<TidyScore> {
        let str_filepath = file_path.to_str().unwrap();

        let mut statement = self
            .connection_pool
            .prepare(
                "SELECT tidy_scores.misnamed, tidy_scores.duplicated, tidy_scores.unused
            FROM my_files
            INNER JOIN tidy_scores ON my_files.tidy_score = tidy_scores.id
            WHERE my_files.path = ?1",
            )
            .unwrap();

        statement.query_row(params![&str_filepath], |row| {
            Ok(TidyScore {
                misnamed: row.get::<_, bool>(0)?,
                duplicated: Some(
                    self.fetch_duplicated_files(path::PathBuf::from(&str_filepath))
                        .unwrap(),
                ),
                unused: row.get::<_, bool>(2)?,
            })
        })
    }

    // The Error type will be changed to something custom in future work on the my_files error handling
    pub fn set_tidyscore(
        &self,
        file_path: PathBuf,
        tidy_score: &TidyScore,
    ) -> Result<(), rusqlite::Error> {
        let str_filepath = file_path.to_str().unwrap();

        let mut statement = self.connection_pool.prepare(
            "INSERT INTO tidy_scores (misnamed, duplicated, unused)
            VALUES (?1, ?2, ?3)",
        )?;

        let duplicated_score = match &tidy_score.duplicated {
            Some(duplicated) => !duplicated.is_empty(),
            None => false,
        };

        let tidy_score_id = statement.insert(params![
            tidy_score.misnamed,
            duplicated_score,
            tidy_score.unused
        ])?;

        let mut statement = self.connection_pool.prepare(
            "UPDATE my_files
            SET tidy_score = ?1
            WHERE path = ?2",
        )?;
        // The potential failure of this query will be handled in future work on the my_files error handling
        let _ = match statement.execute(params![tidy_score_id, str_filepath]) {
            Ok(_) => Ok(info!("tidy_score set for file {:?}", str_filepath)),
            Err(error) => {
                error!(
                    "Error setting tidy_score for file {:?}: {}",
                    file_path, error
                );
                Err(error)
            }
        };
        Ok(())
    }

    pub fn raw_select_query(&self, query: &str, params: &[&dyn ToSql]) -> Result<Vec<FileInfo>> {
        let mut statement = self.connection_pool.prepare(query)?;

        let db_result = statement.query_map(params, MyFiles::create_fileinfo_from_row)?;
        Ok(db_result
            .map(|file| file.unwrap())
            .collect::<Vec<FileInfo>>())
    }

    pub fn raw_query(&self, query: String, params: &[&dyn ToSql]) -> Result<usize> {
        self.connection_pool.execute(query.as_str(), params)
    }
}

// region: --- MyFiles tests
#[cfg(test)]
mod tests {

    use super::*;
    use crate::lister;

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        env_logger::init();
        std::env::set_var("ENV_NAME", "test");
    }

    #[test]
    pub fn main_test() {
        let my_files_builder = MyFilesBuilder::new()
            .configuration_wrapper(ConfigurationWrapper::new().unwrap())
            .seal();
        let my_files = my_files_builder.build().unwrap();
        my_files.init_db().unwrap();

        // region: --- basic tests

        // Checking that there is no file in the database
        assert_eq!(my_files.get_all_files_from_db().unwrap().len(), 0);

        // Adding files to the database
        let directory_path = [r"tests", "assets", "test_folder"].iter().collect();
        lister::list_directories(vec![directory_path])
            .unwrap()
            .iter()
            .for_each(|file| {
                my_files.add_file_to_db(file).unwrap();
            });
        assert_eq!(my_files.get_all_files_from_db().unwrap().len(), 10);

        // Using raw query
        let file_info = match my_files
            .raw_select_query("SELECT * FROM my_files WHERE name = ?1", &[&"test-file-1"])
        {
            Ok(file_info) => file_info,
            Err(error) => {
                assert_eq!(error, rusqlite::Error::QueryReturnedNoRows);
                Vec::new()
            }
        };
        assert_eq!(file_info.len(), 1);
        assert_eq!(file_info[0].name, "test-file-1");
        assert_eq!(file_info[0].size, 100);

        let bad_file_info = match my_files
            .raw_select_query("SELECT * FROM my_files WHERE name = ?1", &[&"xaaaaa"])
        {
            Ok(file_info) => file_info,
            Err(error) => {
                assert_eq!(error, rusqlite::Error::QueryReturnedNoRows);
                Vec::new()
            }
        };
        assert_eq!(bad_file_info.len(), 0);

        // endregion: --- basic tests

        // region: --- TidyScore tests
        let dummy_score = TidyScore {
            misnamed: true,
            duplicated: None,
            unused: true,
        };
        my_files
            .set_tidyscore(
                [r"tests", "assets", "test_folder", "test-file-1"]
                    .iter()
                    .collect(),
                &dummy_score,
            )
            .unwrap();
        let mut score = my_files
            .get_tidyscore(
                [r"tests", "assets", "test_folder", "test-file-1"]
                    .iter()
                    .collect(),
            )
            .unwrap();
        let is_misnamed = score.misnamed;
        let is_unused = score.unused;
        assert!(is_misnamed);
        assert!(is_unused);

        my_files
            .add_duplicated_file_to_db(
                [r"tests", "assets", "test_folder", "test-file-1"]
                    .iter()
                    .collect(),
                [r"tests", "assets", "test_folder", "test-file-2"]
                    .iter()
                    .collect(),
            )
            .unwrap();
        my_files
            .add_duplicated_file_to_db(
                [r"tests", "assets", "test_folder", "test-file-1"]
                    .iter()
                    .collect(),
                [r"tests", "assets", "test_folder", "test-file-3"]
                    .iter()
                    .collect(),
            )
            .unwrap();
        my_files
            .add_duplicated_file_to_db(
                [r"tests", "assets", "test_folder", "test-file-1"]
                    .iter()
                    .collect(),
                [r"tests", "assets", "test_folder", "test-file-4"]
                    .iter()
                    .collect(),
            )
            .unwrap();
        score = my_files
            .get_tidyscore(
                [r"tests", "assets", "test_folder", "test-file-1"]
                    .iter()
                    .collect(),
            )
            .unwrap();
        let is_duplicated = match score.duplicated {
            Some(duplicated) => !duplicated.is_empty(),
            None => false,
        };
        assert!(is_duplicated);

        // endregion: --- TidyScore tests
    }
}
// endregion: --- MyFiles tests
