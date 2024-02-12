use config::Value;
use lazy_static::lazy_static;
use std::collections::HashMap;
use tracing::warn;

use crate::{
    file_info::{FileInfo, TidyScore},
    my_files::MyFiles,
};

pub fn perished(
    file_info: &FileInfo,
    _my_files: &MyFiles,
    raw_params: HashMap<String, Value>,
) -> TidyScore {
    lazy_static! {
        static ref PERISHED_VALID_UNITS: Vec<&'static str> = vec!["day", "week", "month", "year"];
    }

    let _last_modified = file_info.last_modified;

    let time_string_raw: String = match raw_params.get("max") {
        Some(time_string) => match time_string.clone().into_string() {
            Ok(time_string) => time_string,
            Err(err) => {
                warn!("{}", err);
                return TidyScore::new(false, false, None);
            }
        },
        None => {
            warn!(
                "No time_string provided for perished rule: {}",
                file_info.name
            );
            return TidyScore::new(false, false, None);
        }
    };

    let mut max_time_split = time_string_raw.split(' ');
    let _time_amount = match max_time_split.next() {
        Some(time_amount) => match time_amount.parse::<u64>() {
            Ok(time_amount) => time_amount,
            Err(err) => {
                warn!("{}", err);
                return TidyScore::new(false, false, None);
            }
        },
        None => {
            warn!(
                "No time_amount provided for perished rule: {}",
                file_info.name
            );
            return TidyScore::new(false, false, None);
        }
    };
    let _time_unit = match max_time_split.next() {
        Some(time_unit) => {
            if PERISHED_VALID_UNITS.contains(&time_unit) {
                time_unit
            } else {
                warn!(
                    "Invalid time_unit provided for perished rule: {}",
                    file_info.name
                );
                return TidyScore::new(false, false, None);
            }
        }
        None => {
            warn!(
                "No time_unit provided for perished rule: {}",
                file_info.name
            );
            return TidyScore::new(false, false, None);
        }
    };

    // TODO: Wait til we decide which time representation we use

    TidyScore::new(false, true, None)
}
