use config::Value;
use std::collections::HashMap;
use tracing::{debug, error};

use crate::{
    file_info::{FileInfo, TidyScore},
    my_files::MyFiles,
};

// TODO: Change return type to Result<TidyScore, Error> after implementing error handling
pub fn aply_duplicated(
    candidate: &FileInfo,
    my_files: &MyFiles,
    _raw_params: HashMap<String, Value>,
) -> TidyScore {
    let mut duplicated_files: Vec<FileInfo> = Vec::new();
    let mut tidy_score = candidate.tidy_score.clone();

    let all_files = match my_files.get_all_files_from_db() {
        Ok(files) => files,
        Err(err) => {
            error!(
                "TidyAlgo::tidy_rules::duplicated: Could not get all files from db. {:?}",
                err
            );
            return TidyScore::new(false, false, None);
        }
    };

    for file in all_files {
        if file == *candidate && file.path.clone() != candidate.path.clone() {
            debug!(
                "Found a new duplicated file {:?} {:?} with hashs {} : {}",
                file.path.clone(),
                candidate.path.clone(),
                file.hash.clone().unwrap(),
                candidate.hash.clone().unwrap()
            );
            match my_files.add_duplicated_file_to_db(file.path.clone(), candidate.path.clone()) {
                Ok(_) => duplicated_files.push(file),
                Err(err) => {
                    error!(
                        "TidyAlgo::tidy_rules::duplicated: Could not add duplicated file to db. {:?}",
                        err
                    );
                }
            }
        }
    }
    match tidy_score {
        Some(mut score) => match score.duplicated {
            Some(mut duplicated) => {
                duplicated.append(&mut duplicated_files);
                score.duplicated = Some(duplicated);
                tidy_score = Some(score);
                tidy_score.unwrap()
            }
            None => {
                score.duplicated = Some(duplicated_files);
                tidy_score = Some(score);
                tidy_score.unwrap()
            }
        },
        None => TidyScore::new(false, false, Some(duplicated_files)),
    }
}
