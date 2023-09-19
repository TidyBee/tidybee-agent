use crate::file_info::FileInfo;
use axum::Json;

pub async fn get_heaviest_files() -> Json<Vec<FileInfo>> {
    let files: Vec<FileInfo> = vec![
        FileInfo {
            path: std::path::Path::new("/tmp/file1").to_owned(),
            name: "file1".to_string(),
            size: 19990,
            ..Default::default()
        },
        FileInfo {
            path: std::path::Path::new("/tmp/file2").to_owned(),
            name: "file2".to_string(),
            size: 19000,
            ..Default::default()
        },
    ];
    Json(files)
}
