use serde::Serialize;

#[derive(Serialize)]
struct FileInfo {
    path: String,
    size: u64,
}

pub async fn get_heaviest_files() -> &'static str {
    "Get Heaviest Files"
}