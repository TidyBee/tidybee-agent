use axum::extract::{Json};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Params {
    message: String,
    amount: u32
}

pub async fn get_files() -> Result<axum::Json<Vec<(String, i64)>>, axum::Error> {
    let conn = Connection::open("my_files.db").expect("Failed to open database");

    let mut stmt = conn
        .prepare("SELECT name, size FROM files")
        .expect("Failed to prepare SQL statement");

    let rows = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).expect("Failed to execute query");

    let results: Result<Vec<(String, i64)>, rusqlite::Error> = rows.collect();

    match results {
        Ok(data) => {
            Ok(axum::Json(data))
        }
        Err(err) => {
            eprintln!("Error: {:?}", err);
            Ok(axum::Json(vec![])) // Return an empty JSON array in case of an error
        }
    }
}
