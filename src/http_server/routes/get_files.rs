use axum::extract::{Json};
use serde::{Deserialize, Serialize};
use crate::file_info::FileInfo;

#[derive(Serialize, Deserialize)]
pub struct Params {
    message: String,
    amount: u32
}
//
// pub async fn get_files(Json(payload): Json<Greeting>) -> Json<Greeting> {
//     let greeting = Greeting {
//         message: format!("Amount is : {}", payload.amount),
//         amount: payload.amount,
//     };
//     Json(greeting)
// }
//


pub async fn get_files(Json(payload): Json<Params>) -> Json<Vec<FileInfo>> {
    let files: Vec<FileInfo> = vec![
        FileInfo {
            path: std::path::Path::new("src/http_server/mod.rs").to_owned(),
            name: "mod.rs".to_string(),
            size: 73,
            ..Default::default()
        },
        FileInfo {
            path: std::path::Path::new("src/http_server/http_server.rs").to_owned(),
            name: "src/http_server/http_server.rs".to_string(),
            size: 2525,
            ..Default::default()
        },
        FileInfo {
            path: std::path::Path::new("src/http_server/routes/get_files.rs").to_owned(),
            name: "get_files.rs".to_string(),
            size: 1518,
            ..Default::default()
        },
        FileInfo {
            path: std::path::Path::new("src/http_server/routes/hello_world.rs").to_owned(),
            name: "hello_world.rs".to_string(),
            size: 279,
            ..Default::default()
        },
    ];
    Json(files)
}