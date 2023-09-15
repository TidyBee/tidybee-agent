use axum::Json;
use axum::handler;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct FileInfo {
    path: String,
    size: u64,
    last_access: u64,
}

// pub struct Params{
//     amount: fn(files: Vec<FileInfo>, amount: usize) -> Vec<FileInfo>,             //  used to return a certain amount of files
//     heaviest: fn(files: Vec<FileInfo>) -> Vec<FileInfo>,                          //  used to return the heaviest files
//     oldest: fn(files: Vec<FileInfo>) -> Vec<FileInfo>,                            //  used to return the oldest files
//     match_string: fn(files: Vec<FileInfo>, string: String) -> Vec<FileInfo>,      //  used to return files that match a string
//     match_path: fn(files: Vec<FileInfo>, path: String) -> Vec<FileInfo>,          //  used to return files that match a path
// }

// impl Params{
//     pub fn new() -> Params{
//         Params{
//             amount: get_amount_files_from_vec,
//             heaviest: get_heaviest_files_from_vec,
//             oldest: get_oldest_files_from_vec,
//             match_string: get_files_that_match_string,
//             match_path: get_files_that_match_path,
//         }
//     }
// }

#[derive(Serialize, Deserialize)]
pub struct Params {
    amount: u32,
    sort: String,
    match_string: String,
    match_path: String,
}


fn get_oldest_files_from_vec(files: Vec<FileInfo>) -> Vec<FileInfo>{
    let mut files: Vec<FileInfo> = files;

    files.sort_by(|a, b| b.last_access.cmp(&a.last_access));
    return files;
}

fn get_amount_files_from_vec(files: Vec<FileInfo>, amount: usize) -> Vec<FileInfo>{
    let mut files: Vec<FileInfo> = files;

    files.sort_by(|a, b| b.size.cmp(&a.size));
    files.truncate(amount);
    return files;
}

fn get_heaviest_files_from_vec(files: Vec<FileInfo>) -> Vec<FileInfo>{
    let mut files: Vec<FileInfo> = files;

    files.sort_by(|a, b| b.size.cmp(&a.size));
    return files;
}

fn get_files_that_match_string(files: Vec<FileInfo>, string: String) -> Vec<FileInfo>{
    let mut files: Vec<FileInfo> = files;

    files.retain(|file| file.path.contains(&string));
    return files;
}

fn get_files_that_match_path(files: Vec<FileInfo>, path: String) -> Vec<FileInfo>{
    let mut files: Vec<FileInfo> = files;

    files.retain(|file| file.path.contains(&path));
    return files;
}

pub async fn get_files(Json(payload): Json<Params>) -> Json<Vec<FileInfo>> {

    let files: Vec<FileInfo> = vec![
    ];

    Json(files)
}
