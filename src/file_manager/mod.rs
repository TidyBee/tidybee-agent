use std::fs;
use std::io;

pub fn file_exists(file_path: &str) -> bool {
    fs::metadata(file_path).is_ok()
}

pub fn delete_file(file_path: &str) -> Result<(), io::Error> {
    match fs::remove_file(file_path) {
        Ok(()) => Ok(()),
        Err(err) => Err(err),
    }
}
