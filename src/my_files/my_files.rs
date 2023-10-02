use crate::configuration_wrapper::ConfigurationWrapper;
use crate::file_info::FileInfo;
use chrono::{DateTime, Utc};
use log::{error, info};
use rusqlite::{params, Connection, Result, ToSql};
use serde::{Deserialize, Serialize};

// region: --- MyFiles builder states
#[derive(Default, Clone)]
pub struct Sealed;
#[derive(Default, Clone)]
pub struct NotSealed;
#[derive(Default, Clone)]
pub struct NoConfigurationWrapper;
#[derive(Default, Clone)]
pub struct ConfigurationWrapperPresent(ConfigurationWrapper);
// endregion: --- MyFiles builder states

#[derive(Serialize, Deserialize, Default)]
struct MyFilesDatabaseConfiguration {
    pub db_path: String,
    pub drop_db_on_start: bool,
}

pub struct MyFiles {
    connection: Connection,
    configuration: MyFilesDatabaseConfiguration,
}

#[derive(Default, Clone)]
pub struct MyFilesBuilder<C, S> {
    configuration_wrapper_instance: C,
    marker_seal: std::marker::PhantomData<S>,
}

impl MyFilesBuilder<NoConfigurationWrapper, NotSealed> {
    pub fn new() -> Self {
        MyFilesBuilder {
            configuration_wrapper_instance: NoConfigurationWrapper::default(),
            marker_seal: std::marker::PhantomData,
        }
    }
}

impl<C> MyFilesBuilder<C, NotSealed> {
    pub fn configuration_wrapper(self, configuration_wrapper_instance: impl Into<ConfigurationWrapper>) -> MyFilesBuilder<ConfigurationWrapperPresent, NotSealed> {
        MyFilesBuilder {
            configuration_wrapper_instance: ConfigurationWrapperPresent(configuration_wrapper_instance.into()),
            marker_seal: std::marker::PhantomData,
        }
    }
    pub fn seal(self) -> MyFilesBuilder<C, Sealed> {
        MyFilesBuilder {
            configuration_wrapper_instance: self.configuration_wrapper_instance,
            marker_seal: std::marker::PhantomData,
        }
    }
}

impl<S> MyFilesBuilder<ConfigurationWrapperPresent, S> {
    pub fn build(self) -> Result<MyFiles> {
        Ok(MyFiles::new(self.configuration_wrapper_instance.0)?)
    }
}

impl MyFiles {
    pub fn new(config: ConfigurationWrapper) -> Result<Self> {
        let my_files_database_configuration = config
            .bind::<MyFilesDatabaseConfiguration>("my_files_database_configuration")
            .unwrap();

        let connection = Connection::open(my_files_database_configuration.db_path.clone())?;

        Ok(MyFiles { connection, configuration: my_files_database_configuration })
    }
    pub fn init_db(&self) -> Result<(), rusqlite::Error> {
        if self.configuration.drop_db_on_start {
            let drop_db = match self.connection.execute_batch(
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
        match self.connection.execute_batch(
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
            },
            Err(error) => {
                error!("Error initializing database: {}", error);
                Err(error)
            }
        }
    }
    pub fn remove_file_from_db(&self, file_path: &str) -> Result<()> {
        match self.connection.execute(
            "DELETE FROM my_files WHERE path = ?1",
            params![file_path],
        ) {
            Ok(_) => {
                info!("{} removed from my_files", file_path);
                Ok(())
            },
            Err(error) => {
                error!("Error removing {} from my_files: {}", file_path, error);
                Err(error)
            },
        }
    }
    pub fn add_file_to_db(&self, file: &FileInfo) -> Result<()> {
        let last_modified: DateTime<Utc> = file.last_modified.into();
        match self.connection.execute(
            "INSERT INTO my_files (name, path, size, last_modified, tidy_score)
                  VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                file.name,
                file.path.to_str(),
                file.size,
                last_modified.to_string(),
                file.tidy_score.as_ref()
            ],
        ) {
            Ok(_) => Ok(
                info!("{} added to my_files", file.path.to_str().unwrap()),
            ),
            Err(error) => {
                error!("Error adding {} to my_files: {}", file.path.to_str().unwrap(), error);
                Err(error)
            },
        }
    }
    pub fn raw_query(&self, query: String, params: &[&dyn ToSql]) -> Result<usize> {
        self.connection.execute(query.as_str(), params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn init_db() {
        let my_files = MyFiles::new(ConfigurationWrapper::new().unwrap()).unwrap();
        my_files.init_db().unwrap();
    }
}
