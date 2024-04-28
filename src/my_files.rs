#![allow(deprecated)]

use crate::configuration::MyFilesConfiguration;
use crate::file_info::FileInfo;
use chrono::{DateTime, Utc};
use core::marker::PhantomData;
use itertools::{Either, Itertools};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Result, ToSql};
use std::path::PathBuf;
use tracing::{error, info, warn};

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
                last_accessed   DATE NOT NULL
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
            "INSERT INTO my_files (pretty_path, path, size, hash, last_modified, last_accessed)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                file.pretty_path.to_str(),
                file.path.to_str(),
                file.size,
                file.hash,
                last_modified.to_rfc3339(),
                last_accessed.to_rfc3339()
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

        Ok(FileInfo {
            pretty_path: row.get::<_, String>(1)?.into(),
            path,
            size: row.get::<_, u64>(3)?,
            hash: row.get::<_, Option<String>>(4)?,
            last_modified,
            last_accessed,
        })
    }

    pub fn get_all_files_from_db(&self) -> Result<Vec<FileInfo>> {
        let mut statement = self.connection_pool.prepare("SELECT * FROM my_files")?;
        let file_iter_res = statement.query_map(params![], |row| {
            MyFiles::create_fileinfo_from_row(self, row)
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
    use super::*;
    use crate::{configuration, file_lister};

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        env_logger::init();
        std::env::set_var("TIDY_ENV", "test");
    }

    #[test]
    pub fn main_test() {
        let config = configuration::Configuration::init().unwrap();
        let my_files_builder = MyFilesBuilder::new()
            .configure(config.my_files_config)
            .seal();
        let my_files = my_files_builder.build().unwrap();
        my_files.init_db().unwrap();

        // region: --- basic tests

        // Checking that there is no file in the database
        assert_eq!(my_files.get_all_files_from_db().unwrap().len(), 0);

        // Adding files to the database
        let directory_path = vec![[r"tests", "assets", "test_folder"].iter().collect()];
        file_lister::list_directories(directory_path)
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
    }
}
// endregion: --- MyFiles tests
