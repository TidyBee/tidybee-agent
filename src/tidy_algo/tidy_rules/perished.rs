use std::collections::HashMap;

use config::Value;

use crate::{
    file_info::{FileInfo, TidyScore},
    my_files::MyFiles,
};

pub fn perished(file_info: &FileInfo, my_files: &MyFiles, raw_params: HashMap<String, Value>) -> TidyScore {
    TidyScore::new(false, false, None)
}
