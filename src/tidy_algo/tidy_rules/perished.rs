use crate::{
    file_info::{FileInfo, TidyScore},
    my_files::MyFiles,
};

pub fn perished(file_info: &FileInfo, my_files: &MyFiles) -> TidyScore {
    TidyScore::new(false, false, None)
}
