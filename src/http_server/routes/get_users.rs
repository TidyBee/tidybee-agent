use crate::file_info::FileInfo;
use crate::http_server::MyFilesState;
use axum::{extract::State, Json};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct User {
    pub name: String,
    pub username: String,
}

pub async fn get_users(State(my_files): State<MyFilesState>) -> Json<Vec<FileInfo>> {
    let files = my_files
        .my_files
        .lock()
        .unwrap()
        .get_all_files_from_db()
        .unwrap();

    Json(files)
}
