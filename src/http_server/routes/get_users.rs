use std::sync::{Arc, Mutex};
use axum::{Json, extract::State, debug_handler};
use serde::Serialize;
use crate::http_server::MyFilesState;
use crate::my_files::MyFiles;

#[derive(Serialize, Clone)]
pub struct User {
    pub name: String,
    pub username: String,
}


pub async fn get_users(State(my_files): State<MyFilesState>) -> Json<Vec<User>> {
    let files = my_files.my_files.lock().unwrap().get_all_files_from_db().unwrap();

    for file in files {
        println!("file = {:?}", file);
    }

    let users: Vec<User> = vec![
        User {
            name: "Alice".to_string(),
            username: "alice".to_string(),
        },
        User {
            name: "Bob".to_string(),
            username: "bob".to_string(),
        },
    ];
    Json(users)
}
