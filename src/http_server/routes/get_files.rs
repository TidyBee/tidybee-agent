use axum::{Json, http};
use rusqlite::Connection;

pub async fn get_files() -> Result<Json<Vec<(String, i64)>>, http::StatusCode> {
    let conn = Connection::open("my_files.db").expect("Failed to open database");

    let mut stmt = conn
        .prepare("SELECT name, size FROM my_files")
        .expect("Failed to prepare SQL statement");

    let rows = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).expect("Failed to execute query");

    let results: Result<Vec<(String, i64)>, rusqlite::Error> = rows.collect();

    match results {
        Ok(data) => {
            Ok(Json(data))
        }
        Err(err) => {
            eprintln!("Error: {:?}", err);
            Ok(Json(vec![]))
        }
    }
}
