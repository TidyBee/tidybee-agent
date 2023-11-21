use crate::file_info::FileInfo;
use crate::http_server::MyFilesState;
use crate::my_files::MyFiles;
use axum::{extract::State, Json};
use log::error;

struct MyQueryParams {
    sort_by: String,
    number_of_files: usize,
}

pub async fn get_files(State(my_files): State<MyFilesState>) -> Json<Vec<FileInfo>> {
    let files_vec: Vec<FileInfo> = my_files
        .my_files
        .lock()
        .unwrap()
        .get_all_files_from_db()
        .unwrap();
    // let sort_by = params.sort_by.to_lowercase();

    // files_vec.sort_by(|a, b| {
    //     match sort_by.as_str() {
    //         "size" => b.size.cmp(&a.size),
    //         "lastupdate" => b.last_modified.cmp(&a.last_modified),
    //         _ => {
    //             error!("Invalid sort parameter in get_files route. Defaulting to size.");
    //             b.size.cmp(&a.size)
    //         }
    //     }
    // });
    // let result = files_vec.into_iter().take(params.number_of_files).collect();
    Json(files_vec)
}
