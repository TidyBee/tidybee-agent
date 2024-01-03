use std::collections::HashMap;

use config::Value;
use log::{error, info};

use crate::{
    file_info::{FileInfo, TidyScore},
    my_files::MyFiles,
};

// TODO: Change return type to Result<TidyScore, Error> after implementing error handling
pub fn duplicated(candidate: &FileInfo, my_files: &MyFiles, raw_params: HashMap<String, Value>) -> TidyScore {
    let mut duplicated_files: Vec<FileInfo> = Vec::new();
    let mut tidy_score = candidate.tidy_score.clone();

    let all_files = match my_files.get_all_files_from_db() {
        Ok(files) => files,
        Err(err) => {
            error!("TidyAlgo::tidy_rules::duplicated: Could not get all files from db. {:?}", err);
            return TidyScore::new(false, false, None);
        },
    };

    for file in all_files {
        if file == *candidate {
            if my_files.add_duplicated_file_to_db(file.path.clone(), candidate.path.clone()).is_err() {
                error!("TidyAlgo::tidy_rules::duplicated: Could not add duplicated file to db. {:?}", file.path);
            } else {
                duplicated_files.push(file);
            }
        }
    }
    match tidy_score {
        Some(mut score) => {
            match score.duplicated {
                Some(mut duplicated) => {
                    duplicated.append(&mut duplicated_files);
                    score.duplicated = Some(duplicated);
                    tidy_score = Some(score);
                    tidy_score.unwrap()
                },
                None => {
                    score.duplicated = Some(duplicated_files);
                    tidy_score = Some(score);
                    tidy_score.unwrap()
                },

            }
        },
        None => {
            TidyScore::new(false, false, Some(duplicated_files))
        },
    }
}
