use config::Value;
use regex::Regex;
use std::collections::HashMap;
use tracing::warn;

use crate::{
    file_info::{FileInfo, TidyScore},
    my_files::MyFiles,
};

pub fn misnamed(
    file_info: &FileInfo,
    _my_files: &MyFiles,
    raw_params: HashMap<String, Value>,
) -> TidyScore {
    let pattern = raw_params.get("pattern");
    let mut new_score = match file_info.tidy_score.clone() {
        Some(score) => score,
        None => TidyScore::new(false, false, None),
    };

    if pattern.is_none() {
        warn!("No pattern provided for misnamed rule");
        return new_score.clone();
    } else {
        let pattern_str = match pattern.unwrap().clone().into_string() {
            Ok(pattern) => pattern,
            Err(_) => {
                warn!("Pattern provided for misnamed rule is not a string");
                return new_score;
            }
        };

        let re = Regex::new(pattern_str.as_str()).unwrap();
        if re.is_match(&file_info.pretty_path) {
            return new_score;
        } else {
            new_score.misnamed = true;
            return new_score;
        }
    }
}
