use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct FileInfo {
    path: String,
    size: u64,
}

pub async fn get_heaviest_files() -> Json<Vec<FileInfo>> {
    let files: Vec<FileInfo> = vec![
        FileInfo {
            path: "/tmp/file1".to_string(),
            size: 19990,
        },
        FileInfo {
            path: "/tmp/file2".to_string(),
            size: 19000,
        },
    ];
    Json(files)
}
