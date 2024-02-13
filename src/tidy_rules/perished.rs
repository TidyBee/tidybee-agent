use chrono::{DateTime, Utc};
use config::Value;
use lazy_static::lazy_static;
use std::collections::HashMap;
use tracing::warn;

use crate::{
    file_info::{FileInfo, TidyScore},
    my_files::MyFiles,
};

pub fn apply_perished(
    file_info: &FileInfo,
    _my_files: &MyFiles,
    raw_params: HashMap<String, Value>,
) -> TidyScore {
    let mut tidy_score = file_info.tidy_score.clone();
    let max_retention_date_string = match raw_params.get("max") {
        Some(s) => match s.clone().into_string() {
            Ok(s) => s,
            Err(e) => {
                warn!("Error parsing max date: {}", e);
                return TidyScore::new(false, false, None);
            }
        },
        _ => {
            warn!("No max date provided");
            return TidyScore::new(false, false, None);
        }
    };
    let max_retention_date: DateTime<Utc> = match max_retention_date_string.parse() {
        Ok(d) => d,
        Err(e) => {
            warn!("Error parsing max date: {}", e);
            return TidyScore::new(false, false, None);
        }
    };
    let last_accessed: DateTime<Utc> = file_info.last_accessed.into();
    let perished: bool = last_accessed < max_retention_date;

    match tidy_score {
        Some(mut score) => {
            score.unused = perished;
            tidy_score = Some(score);
            tidy_score.unwrap()
        }
        None => {
            if last_accessed < max_retention_date {
                TidyScore::new(false, false, None)
            } else {
                TidyScore::new(false, true, None)
            }
        }
    }
}
