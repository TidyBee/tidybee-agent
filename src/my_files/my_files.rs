use crate::configuration_wrapper::ConfigurationWrapper;
use crate::file_info::FileInfo;
use chrono::{DateTime, Utc};
use log::{error, info, warn};
use r2d2;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Result, ToSql};
use serde::{Deserialize, Serialize};

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
                DROP TABLE IF EXISTS my_files;
                DROP TABLE IF EXISTS tidy_scores;
                DROP TABLE IF EXISTS duplicates_associative_table;
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
                last_modified   DATE NOT NULL,
                tidy_score      INTEGER UNIQUE,
                FOREIGN KEY (tidy_score) REFERENCES tidy_scores(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS tidy_scores (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                misnamed        BOOLEAN NOT NULL,
                misplaced       BOOLEAN NOT NULL,
                unused          BOOLEAN NOT NULL
            );
            CREATE TABLE IF NOT EXISTS duplicates_associative_table (
                tidy_score_id   INTEGER NOT NULL,
                my_file_id      INTEGER NOT NULL,
                PRIMARY KEY (tidy_score_id, my_file_id),
                FOREIGN KEY (tidy_score_id) REFERENCES tidy_scores(id),
                FOREIGN KEY (my_file_id) REFERENCES my_files(id) ON DELETE CASCADE
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
    pub fn add_file_to_db(&self, file: &FileInfo) -> Result<()> {
        let last_modified: DateTime<Utc> = file.last_modified.into();
        match self.connection_pool.execute(
            "INSERT INTO my_files (name, path, size, last_modified, tidy_score)
                  VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                file.name,
                file.path.to_str(),
                file.size,
                last_modified.to_rfc3339(),
                file.tidy_score.as_ref()
            ],
        ) {
            Ok(_) => Ok(info!("{} added to my_files", file.path.to_str().unwrap())),
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
    pub fn get_all_files_from_db(&self) -> Result<Vec<FileInfo>> {
        let mut statement = self.connection_pool.prepare("SELECT * FROM my_files")?;
        let file_iter = statement.query_map(params![], |row| {
            let path_str = row.get::<_, String>(2)?;
            let path = std::path::Path::new(&path_str).to_owned();

            let time_str = row.get::<_, String>(4)?;
            let last_modified = match DateTime::parse_from_rfc3339(&time_str) {
                Ok(last_modified) => last_modified.into(),
                Err(error) => {
                    error!(
                        "Error parsing key: last_modified with value {}, for file {}. {}",
                        path_str, time_str, error
                    );
                    std::time::SystemTime::UNIX_EPOCH
                }
            };

            Ok(FileInfo {
                name: row.get::<_, String>(1)?,
                path,
                size: row.get::<_, u64>(3)?,
                last_modified,
                tidy_score: row.get(5)?,
            })
        })?;
        let mut files_vec: Vec<FileInfo> = Vec::new();
        for file in file_iter {
            files_vec.push(file.unwrap());
        }
        Ok(files_vec)
    }

    pub fn raw_select_query(&self, query: &str, params: &[&dyn ToSql]) -> Result<Vec<FileInfo>> {
        let mut statement = self.connection_pool.prepare(query)?;

        let db_result = statement.query_map(params, |row| {
            Ok(FileInfo {
                name: row.get::<_, String>(1)?,
                path: std::path::Path::new(row.get::<_, String>(2)?.as_str()).to_owned(),
                size: row.get::<_, u64>(3)?,
                last_modified: row
                    .get::<_, String>(4)?
                    .parse::<DateTime<Utc>>()
                    .unwrap()
                    .into(),
                tidy_score: row.get(5)?,
            })
        })?;
        Ok(db_result
            .map(|file| file.unwrap())
            .collect::<Vec<FileInfo>>())
    }

    pub fn raw_query(&self, query: String, params: &[&dyn ToSql]) -> Result<usize> {
        self.connection_pool.execute(query.as_str(), params)
    }
}

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

        // Checking that there is no file in the database
        assert_eq!(my_files.get_all_files_from_db().unwrap().len(), 0);

        // Adding files to the database
        lister::list_directories(vec!["./tests/assets/test_folder".into()])
            .unwrap()
            .iter()
            .for_each(|file| {
                my_files.add_file_to_db(file).unwrap();
            });
        assert_eq!(my_files.get_all_files_from_db().unwrap().len(), 10);

        // Using raw query
        let file_info = my_files
            .raw_select_query("SELECT * FROM my_files WHERE name = ?1", &[&"test-file-1"])
            .unwrap();
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
    }
}
