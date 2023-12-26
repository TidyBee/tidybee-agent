use crate::file_info::{FileInfo, TidyScore};

pub fn duplicated(file_info: &FileInfo) -> TidyScore {
    TidyScore::new(false, false, None)
}