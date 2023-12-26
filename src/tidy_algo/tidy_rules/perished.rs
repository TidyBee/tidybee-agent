use crate::file_info::{FileInfo, TidyScore};

pub fn perished(file_info: &FileInfo) -> TidyScore {
    TidyScore::new(false, false, None)
}