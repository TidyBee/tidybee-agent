use crate::configuration::MyFilesConfiguration;
use crate::file_info::{FileInfo, TidyScore};
use crate::tidy_algo::TidyAlgo;
use chrono::{DateTime, Utc};
use core::marker::PhantomData;
use itertools::{Either, Itertools};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Result, ToSql};
use std::path;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

// region: --- MyFiles builder states
#[derive(Default, Clone)]
pub struct Sealed;

#[derive(Default, Clone)]
pub struct NotSealed;

#[derive(Default, Clone)]
pub struct NoConfiguration;

#[derive(Default, Clone)]
pub struct ConfigurationPresent(MyFilesConfiguration);

#[derive(Default, Clone)]
pub struct NoConnectionManager;

#[derive(Clone)]
pub struct ConnectionManagerPresent(Pool<SqliteConnectionManager>);
// endregion: --- MyFiles builder states

pub struct MyFiles {
    connection_pool: PooledConnection<SqliteConnectionManager>,
    configuration: MyFilesConfiguration,
}

#[derive(Copy, Clone, Default)]
pub struct MyFilesBuilder<C, M, S> {
    connection_manager: M,
    configuration_instance: C,
    marker_seal: PhantomData<S>,
}

impl Default for ConnectionManagerPresent {
    fn default() -> Self {
        ConnectionManagerPresent(Pool::new(SqliteConnectionManager::file("my_files.db")).unwrap())
    }
}

impl MyFilesBuilder<NoConfiguration, NoConnectionManager, NotSealed> {
    pub const fn new() -> Self {
        MyFilesBuilder {
            connection_manager: NoConnectionManager,
            configuration_instance: NoConfiguration,
            marker_seal: PhantomData,
        }
    }
}

impl<C, M> MyFilesBuilder<C, M, NotSealed> {
    pub fn configure(
        self,
        configuration_instance: MyFilesConfiguration,
    ) -> MyFilesBuilder<ConfigurationPresent, ConnectionManagerPresent, NotSealed> {
        let connection_manager =
            SqliteConnectionManager::file(configuration_instance.db_path.clone());
        let pool = match Pool::new(connection_manager) {
            Ok(pool) => pool,
            Err(error) => {
                error!("Error creating connection pool: {}", error);
                panic!();
            }
        };
        MyFilesBuilder {
            connection_manager: ConnectionManagerPresent(pool),
            configuration_instance: ConfigurationPresent(configuration_instance),
            marker_seal: PhantomData,
        }
    }
    pub fn seal(self) -> MyFilesBuilder<C, M, Sealed> {
        MyFilesBuilder {
            connection_manager: self.connection_manager,
            configuration_instance: self.configuration_instance,
            marker_seal: PhantomData,
        }
    }
}

impl MyFilesBuilder<ConfigurationPresent, ConnectionManagerPresent, Sealed> {
    pub fn build(&self) -> Result<MyFiles> {
        let my_files_config = self.configuration_instance.0.clone();
        let connection_pool = match self.connection_manager.0.get() {
            Ok(connection) => connection,
            Err(error) => {
                error!("Error getting connection from pool: {}", error);
                panic!();
            }
        };
        MyFiles::new(my_files_config, connection_pool)
    }
}

#[allow(dead_code)]
impl MyFiles {
    pub fn new(
        configuration: MyFilesConfiguration,
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
                pretty_path     TEXT NOT NULL UNIQUE,
                path            TEXT NOT NULL UNIQUE,
                size            INTEGER NOT NULL,
                hash            TEXT DEFAULT \"\",
                last_modified   DATE NOT NULL,
                last_accessed   DATE NOT NULL,
                tidy_score_id      INTEGER UNIQUE,
                FOREIGN KEY (tidy_score_id) REFERENCES tidy_scores(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS tidy_scores (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                misnamed        BOOLEAN NOT NULL,
                unused          BOOLEAN NOT NULL,
                duplicated      BOOLEAN NOT NULL,
                grade           INTEGER DEFAULT 0
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

    pub fn remove_file_from_db(&self, file_path: PathBuf) -> Result<()> {
        let str_filepath = file_path.to_str().unwrap();

        match self.connection_pool.execute(
            "DELETE FROM my_files WHERE path = ?1",
            params![str_filepath],
        ) {
            Ok(_) => {
                info!("{} removed from my_files", str_filepath);
                Ok(())
            }
            Err(error) => {
                error!("Error removing {} from my_files: {}", str_filepath, error);
                Err(error)
            }
        }
    }

    pub fn add_file_to_db(&self, file: &FileInfo) -> Result<FileInfo> {
        let last_modified: DateTime<Utc> = file.last_modified.into();
        let last_accessed: DateTime<Utc> = file.last_accessed.into();
        match self.connection_pool.execute(
            "INSERT INTO my_files (pretty_path, path, size, hash, last_modified, last_accessed, tidy_score_id)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                file.pretty_path.to_str(),
                file.path.to_str(),
                file.size,
                file.hash,
                last_modified.to_rfc3339(),
                last_accessed.to_rfc3339(),
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
                Err(error)
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
                "SELECT id, tidy_score_id FROM my_files
                WHERE path = ?1",
            )
            .unwrap();
        let result = statement
            .query_row(params![&str_filepath], |row| {
                Ok((
                    row.get::<_, i64>("id"),
                    row.get::<_, Option<i64>>("tidy_score_id"),
                ))
            })
            .unwrap();
        let file_id: i64 = result.0?;
        let tidy_score_id: Option<i64> = result.1?;

        let mut statement = self
            .connection_pool
            .prepare(
                "SELECT id FROM my_files
            WHERE path = ?1",
            )
            .unwrap();
        let duplicated_file_id: Option<i64> = statement
            .query_row(
                params![duplicated_file_path.into_os_string().to_str()],
                |row| row.get::<_, Option<i64>>(0),
            )
            .unwrap();

        let mut statement = self
            .connection_pool
            .prepare(
                "INSERT INTO duplicates_associative_table (original_file_id, duplicated_file_id)
            VALUES (?1, ?2)",
            )
            .unwrap();
        let _: Result<(), _> = match statement.execute(params![file_id, duplicated_file_id]) {
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

    fn create_fileinfo_from_row(&self, row: &rusqlite::Row) -> Result<FileInfo> {
        let path_str = row.get::<_, String>(2)?;
        let path = std::path::Path::new(&path_str).to_owned();

        let mut time_str = row.get::<_, String>(5)?;
        if DateTime::parse_from_rfc3339(&time_str).is_err() {
            error!(
                "create_fileinfo_from_row: Error parsing key: last_modified with value {}, for file {}.",
                time_str, path_str
            );
        }
        let last_modified = DateTime::parse_from_rfc3339(&time_str).unwrap().into();

        time_str = row.get::<_, String>(6)?;
        let last_accessed = match DateTime::parse_from_rfc3339(&time_str) {
            Ok(last_accessed) => last_accessed.into(),
            Err(error) => {
                error!(
                    "create_fileinfo_from_row: Error parsing key: last_accessed with value {}, for file {}. {}",
                    time_str, path_str, error
                );
                std::time::SystemTime::UNIX_EPOCH
            }
        };

        let tidy_score = match row.get::<_, Option<i64>>("tidy_score_id") {
            Ok(tidy_score_id) => match tidy_score_id {
                Some(tidy_score_id) => {
                    match self.get_tidyscore_from_id(tidy_score_id, path.clone()) {
                        Ok(score) => Some(score),
                        Err(error) => {
                            error!(
                            "create_fileinfo_from_row: Error getting tidy_score_id: {} for file: {} : {}",
                            tidy_score_id, path_str, error
                        );
                            None
                        }
                    }
                }
                None => {
                    debug!(
                        "create_fileinfo_from_row: No TidyScore was created yet for {}",
                        path_str
                    );
                    None
                }
            },
            Err(error) => {
                error!(
                    "create_fileinfo_from_row: Error parsing key: tidy_score_id for file {} : {}",
                    path_str, error
                );
                None
            }
        };

        Ok(FileInfo {
            pretty_path: row.get::<_, String>(1)?.into(),
            path,
            size: row.get::<_, u64>(3)?,
            hash: row.get::<_, Option<String>>(4)?,
            last_modified,
            last_accessed,
            tidy_score,
        })
    }

    pub fn get_all_files_from_db(&self) -> Result<Vec<FileInfo>> {
        let mut statement = self.connection_pool.prepare("SELECT * FROM my_files")?;
        let file_iter_res = statement.query_map(params![], |row| {
            MyFiles::create_fileinfo_from_row(self, &row)
        });

        let file_iter = match file_iter_res {
            Ok(file_iter) => file_iter,
            Err(error) => {
                error!(
                    "get_all_files_from_db: Error getting file iterator from database: {}",
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
                "get_all_files_from_db: Error getting file from database: {:?}",
                err
            );
        }
        Ok(files_vec)
    }

    pub fn fetch_duplicated_files(&self, file_path: PathBuf) -> Result<Vec<FileInfo>> {
        let str_file_path = file_path.to_str().unwrap();

        // Check if file is duplicated
        let mut duplicate_check_stmt = self.connection_pool.prepare(
            "SELECT duplicated FROM tidy_scores INNER JOIN my_files ON tidy_scores.id = my_files.tidy_score_id WHERE my_files.path = ?1",
        )?;
        let duplicated =
            duplicate_check_stmt.query_row(params![&str_file_path], |row| row.get::<_, bool>(0))?;

        if !duplicated {
            debug!("No duplicate found while checking {:?}", file_path);
            return Ok(Vec::new());
        }

        let mut statement = self
            .connection_pool
            .prepare("SELECT id FROM my_files WHERE path = ?1")?;
        let file_id = statement.query_row(params![&str_file_path], |row| row.get::<_, i64>(0))?;

        let mut statement = self.connection_pool.prepare(
            "SELECT my_files.pretty_path, my_files.path, my_files.size, my_files.last_modified, my_files.last_accessed, my_files.hash, my_files.tidy_score_id
            FROM my_files
            INNER JOIN duplicates_associative_table ON my_files.id = duplicates_associative_table.original_file_id WHERE duplicates_associative_table.original_file_id = ?1",
        ).unwrap();

        let duplicated_files = statement
            .query_map(params![file_id], |row| {
                let path_str = row.get::<_, String>("pretty_path")?;
                let path = std::path::Path::new(&path_str).to_owned();

                let mut time_str = row.get::<_, String>("last_modified")?;
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

                time_str = row.get::<_, String>("last_accessed")?;
                let last_accessed = match DateTime::parse_from_rfc3339(&time_str) {
                    Ok(last_accessed) => last_accessed.into(),
                    Err(error) => {
                        error!(
                            "Error parsing key: last_accessed with value {}, for file {}. {}",
                            time_str, path_str, error
                        );
                        std::time::SystemTime::UNIX_EPOCH
                    }
                };

                let tidy_score_id = row.get::<_, i64>("tidy_score_id")?;
                let mut statement = self.connection_pool.prepare(
                    "SELECT misnamed, duplicated, unused, grade FROM tidy_scores WHERE id = ?1",
                )?;
                let tidy_score = statement.query_row(params![tidy_score_id], |row| {
                    Ok(TidyScore {
                        misnamed: row.get::<_, bool>("misnamed")?,
                        duplicated: None,
                        unused: row.get::<_, bool>("unused")?,
                        grade: row.get::<_, Option<u8>>("grade")?,
                    })
                })?;

                Ok(FileInfo {
                    pretty_path: row.get::<_, String>("pretty_path")?.into(),
                    path,
                    size: row.get::<_, u64>("size")?,
                    hash: row.get::<_, Option<String>>("hash")?,
                    last_modified,
                    last_accessed,
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
                "SELECT tidy_scores.misnamed, tidy_scores.duplicated, tidy_scores.unused, tidy_scores.grade
            FROM my_files
            INNER JOIN tidy_scores ON my_files.tidy_score_id = tidy_scores.id
            WHERE my_files.path = ?1",
            )
            .unwrap();

        statement.query_row(params![&str_filepath], |row| {
            Ok(TidyScore {
                misnamed: row.get::<_, bool>("misnamed")?,
                duplicated: Some(
                    self.fetch_duplicated_files(path::PathBuf::from(&str_filepath))
                        .unwrap(),
                ),
                unused: row.get::<_, bool>("unused")?,
                grade: row.get::<_, Option<u8>>("grade")?,
            })
        })
    }

    /// This method shall only be used internally to my_files as it relies on indexes and is not database agnostic
    fn get_tidyscore_from_id(&self, id: i64, file_path: PathBuf) -> Result<TidyScore> {
        let mut statement = self
            .connection_pool
            .prepare("SELECT misnamed, duplicated, unused, grade FROM tidy_scores WHERE id = ?1")?;
        statement.query_row(params![id], |row| {
            Ok(TidyScore {
                misnamed: row.get::<_, bool>("misnamed")?,
                duplicated: Some(self.fetch_duplicated_files(file_path).unwrap()),
                unused: row.get::<_, bool>("unused")?,
                grade: row.get::<_, Option<u8>>("grade")?,
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
        let mut statement: rusqlite::Statement<'_>;
        let duplicated_score = match &tidy_score.duplicated {
            Some(duplicated) => !duplicated.is_empty(),
            None => false,
        };

        statement = self.connection_pool.prepare(
            "SELECT tidy_score_id FROM my_files
            WHERE path = ?1",
        )?;
        let mut current_tidy_score_id: Option<i64> = statement
            .query_row(params![str_filepath], |row| {
                Ok(row.get::<_, Option<i64>>(0)?)
            })?;

        // If the file already has a tidyscore attached to it, we update it
        if current_tidy_score_id.is_some() {
            statement = self.connection_pool.prepare(
                "UPDATE tidy_scores
                SET misnamed = ?1, duplicated = ?2, unused = ?3
                WHERE id = ?4",
            )?;
            statement.execute(params![
                tidy_score.misnamed,
                duplicated_score,
                tidy_score.unused,
                current_tidy_score_id
            ])?;
        } else {
            // If the file doesn't have a tidyscore attached to it, we create one
            statement = self.connection_pool.prepare(
                "INSERT INTO tidy_scores (misnamed, duplicated, unused) VALUES (?1, ?2, ?3)",
            )?;
            current_tidy_score_id = Some(statement.insert(params![
                tidy_score.misnamed,
                duplicated_score,
                tidy_score.unused
            ])?);
            // And we attach it to the file
            statement = self.connection_pool.prepare(
                "UPDATE my_files
                SET tidy_score_id = ?1
                WHERE path = ?2",
            )?;
            // The potential failure of this query will be handled in future work on the my_files error handling
            let _: Result<(), _> =
                match statement.execute(params![current_tidy_score_id, str_filepath]) {
                    Ok(_) => Ok(info!("tidy_score set for file {:?}", str_filepath)),
                    Err(error) => {
                        error!(
                            "Error setting tidy_score for file {:?}: {}",
                            file_path, error
                        );
                        Err(error)
                    }
                };
        }

        Ok(())
    }

    /// Update the grade field based on the content of the tidy_scores table
    /// Does not do anything if the file does not have a tidy_score attached to it
    pub fn update_grade(&self, file_path: PathBuf, tidy_algo: &TidyAlgo) {
        let str_filepath = file_path.to_str().unwrap();
        let mut statement = self
            .connection_pool
            .prepare(
                "SELECT tidy_score_id FROM my_files
            WHERE path = ?1",
            )
            .unwrap();
        let current_tidy_score_id: Option<i64> = statement
            .query_row(params![str_filepath], |row| {
                Ok(row.get::<_, Option<i64>>(0)?)
            })
            .unwrap();

        if current_tidy_score_id.is_none() {
            debug!("No tidy_score found for file {:?}", file_path);
            return;
        }

        let mut statement = self
            .connection_pool
            .prepare(
                "SELECT misnamed, duplicated, unused FROM tidy_scores
            WHERE id = ?1",
            )
            .unwrap();
        let tidy_score = statement
            .query_row(params![current_tidy_score_id], |row| {
                Ok(TidyScore {
                    misnamed: row.get::<_, bool>("misnamed")?,
                    duplicated: Some(self.fetch_duplicated_files(file_path.clone()).unwrap()),
                    unused: row.get::<_, bool>("unused")?,
                    grade: None,
                })
            })
            .unwrap();

        let grade = tidy_algo.compute_grade(&tidy_score);
        let mut statement = self
            .connection_pool
            .prepare(
                "UPDATE tidy_scores
            SET grade = ?1
            WHERE id = ?2",
            )
            .unwrap();
        match statement.execute(params![grade.0, current_tidy_score_id]) {
            Ok(_) => info!("Grade updated for file {:?}", file_path),
            Err(error) => error!("Error updating grade for file {:?}: {}", file_path, error),
        };
    }

    #[deprecated(
        note = "This method used now as a feature patch for developement and will be removed in future versions"
    )]
    pub fn raw_select_query(&self, query: &str, params: &[&dyn ToSql]) -> Result<Vec<FileInfo>> {
        let mut statement = self.connection_pool.prepare(query)?;

        let db_result =
            statement.query_map(params, |row| MyFiles::create_fileinfo_from_row(self, row))?;
        Ok(db_result
            .map(core::result::Result::unwrap)
            .collect::<Vec<FileInfo>>())
    }

    #[deprecated(
        note = "This method used now as a feature patch for developement and will be removed in future versions"
    )]
    pub fn raw_query(&self, query: String, params: &[&dyn ToSql]) -> Result<usize> {
        self.connection_pool.execute(query.as_str(), params)
    }
}

// region: --- MyFiles tests
#[cfg(test)]
mod tests {

    use std::env::current_dir;

    use super::*;
    use crate::{configuration, file_lister, tidy_algo::TidyGrade};

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        env_logger::init();
        std::env::set_var("TIDY_ENV", "test");
    }

    #[test]
    pub fn main_test() {
        let config = configuration::Configuration::init();
        let my_files_builder = MyFilesBuilder::new()
            .configure(config.my_files_config)
            .seal();
        let my_files = my_files_builder.build().unwrap();
        my_files.init_db().unwrap();

        // region: --- basic tests

        // Checking that there is no file in the database
        assert_eq!(my_files.get_all_files_from_db().unwrap().len(), 0);

        // Adding files to the database
        let directory_path = [r"tests", "assets", "test_folder"].iter().collect();
        file_lister::list_directories(vec![directory_path])
            .unwrap()
            .iter()
            .for_each(|file| match my_files.add_file_to_db(file) {
                Ok(_) => (),
                Err(err) => {
                    error!("Error adding file to database: {}", err);
                    panic!();
                }
            });
        assert_eq!(my_files.get_all_files_from_db().unwrap().len(), 13);

        // Using raw query
        let test_file_path: PathBuf = [r"tests", r"assets", r"test_folder", r"test-file-1"]
            .iter()
            .collect();
        let file_info = match my_files.raw_select_query(
            "SELECT * FROM my_files WHERE pretty_path = ?1",
            &[&test_file_path.to_str()],
        ) {
            Ok(file_info) => file_info,
            Err(error) => {
                assert_eq!(error, rusqlite::Error::QueryReturnedNoRows);
                Vec::new()
            }
        };
        assert_eq!(file_info.len(), 1);
        assert_eq!(file_info[0].pretty_path, test_file_path);
        assert_eq!(file_info[0].size, 100);

        let bad_file_info = match my_files.raw_select_query(
            "SELECT * FROM my_files WHERE pretty_path = ?1",
            &[&"xaaaaa"],
        ) {
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
            grade: Some(1),
        };
        let mut tests_dir = current_dir().unwrap();
        tests_dir.push(
            [r"tests", "assets", "test_folder"]
                .iter()
                .collect::<PathBuf>(),
        );

        my_files
            .set_tidyscore(tests_dir.join("test-file-1"), &dummy_score)
            .unwrap();
        let mut score = my_files
            .get_tidyscore(tests_dir.join("test-file-1"))
            .unwrap();
        let is_misnamed = score.misnamed;
        let is_unused = score.unused;
        let grade = score.grade;
        assert!(is_misnamed);
        assert!(is_unused);
        assert_eq!(grade, Some(0));
        assert_eq!(TidyGrade(grade.unwrap()).display_grade(), "A");

        my_files
            .add_duplicated_file_to_db(tests_dir.join("test-file-1"), tests_dir.join("test-file-2"))
            .unwrap();
        my_files
            .add_duplicated_file_to_db(tests_dir.join("test-file-1"), tests_dir.join("test-file-3"))
            .unwrap();
        my_files
            .add_duplicated_file_to_db(tests_dir.join("test-file-1"), tests_dir.join("test-file-4"))
            .unwrap();
        score = my_files
            .get_tidyscore(tests_dir.join("test-file-1"))
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
