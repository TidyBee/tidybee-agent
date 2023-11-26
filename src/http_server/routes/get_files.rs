use crate::file_info::FileInfo;
use crate::http_server::MyFilesState;
use axum::{extract::State, Json};
use axum::extract::Path;
use log::error;

pub async fn get_files(
    State(my_files): State<MyFilesState>,
    Path((number_of_files, sort_by)): Path<(usize, String)>)
    -> Json<Vec<FileInfo>> {
    let mut files_vec: Vec<FileInfo> = my_files
        .my_files
        .lock()
        .unwrap()
        .get_all_files_from_db()
        .unwrap();

    files_vec.sort_by(|a, b| {
        match sort_by.to_lowercase().as_str() {
            "size" => b.size.cmp(&a.size),
            "last_update" => b.last_modified.cmp(&a.last_modified),
            _ => {
                error!("Invalid sort parameter in get_files route. Defaulting to size.");
                b.size.cmp(&a.size)
            }
        }
    });
    let result = files_vec.into_iter().take(number_of_files).collect();
    Json(result)
}
