use crate::configuration_wrapper::ConfigurationWrapper;
use crate::file_info::FileInfo;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
struct MyFilesDatabaseConfiguration {
    pub db_path: String,
    pub drop_db_on_start: bool,
}

pub struct MyFiles {
    connection: Connection,
    configuration: MyFilesDatabaseConfiguration
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
                Ok(_) => Ok(println!("Database dropped")),
                Err(error) => {
                    eprintln!("Error dropping database: {}", error);
                    Err(error)
                },
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
                tidy_score_id INTEGER NOT NULL,
                my_file_id INTEGER NOT NULL,
                FOREIGN KEY (my_file_id, tidy_score_id) REFERENCES my_files (id, tidy_score) ON DELETE CASCADE
            );
            COMMIT;",
        ) {
            Ok(_) => Ok(()),
            Err(error) => {
                eprintln!("Error initializing database: {}", error);
                Err(error)
            },
        }
    }
    // fn associate_file_to_tidyscore(&self, file: FileInfo, tidy_score_rowid: i64) -> Result<()> {
    //     match self.connection.execute(
    //         "INSERT INTO duplicates_associative_table (tidy_score_id, my_file_id) VALUES (?1, ?2)",
    //         params![tidy_score_rowid, file.id],
    //     ) {
    //         Ok(_) => Ok(()),
    //         Err(error) => Err(error),
    //     }
    // }
    pub fn add_file_to_db(&self, file: &FileInfo) -> Result<()> {
        // let mut tidy_score_rowid: Option<i64> = None;
        //
        // if !file.tidy_score.is_none() {
        //     match self.connection.execute(
        //         "INSERT INTO tidy_scores (misnamed, misplaced, unused) VALUES (?1, ?2, ?3)",
        //         params![
        //             file.tidy_score.unwrap().misnamed,
        //             file.tidy_score.unwrap().misplaced,
        //             file.tidy_score.unwrap().unused
        //         ],
        //     ) {
        //         Ok(_) => tidy_score_rowid = Some(self.connection.last_insert_rowid()),
        //         Err(error) => eprintln!("Error inserting tidy score: {}", error),
        //     }
        // }

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
            Ok(_) => Ok(()),
            Err(error) => Err(error),
        }
    }
}
