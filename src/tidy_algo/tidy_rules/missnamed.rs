use crate::file_info::{FileInfo, TidyScore};

pub fn missnamed(file_info: &FileInfo) -> TidyScore {
    TidyScore::new(false, false, None)
}
