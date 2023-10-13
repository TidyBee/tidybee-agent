use std::fs;
use std::io;

pub fn delete_file(file_path: &str) -> Result<(), io::Error> {
    match fs::remove_file(file_path) {
        Ok(()) => Ok(()),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_file_reg() {
        let file_path = "test.txt";
        fs::write(file_path, "").expect("Failed to create file");
        assert!(delete_file(file_path).is_ok());
        assert!(fs::metadata(file_path).is_err());
    }

    #[test]
    fn test_delete_file_dir() {
        let file_path = "test";
        fs::create_dir(file_path).expect("Failed to create dir");
        assert!(delete_file(file_path).is_err());
        fs::remove_dir(file_path).expect("Failed to remove dir");
    }

    #[test]
    fn test_delete_file_does_not_exist() {
        let file_path = "test.txt";
        assert!(delete_file(file_path).is_err());
    }
}
