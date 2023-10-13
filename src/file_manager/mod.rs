use std::fs;
use std::io;

#[allow(dead_code)]
pub fn delete_file(file_path: &str) -> Result<(), io::Error> {
    match fs::remove_file(file_path) {
        Ok(()) => Ok(()),
        Err(err) => Err(err),
    }
}

#[allow(dead_code)]
pub fn move_file(source_file_path: &str, target_file_path: &str) -> Result<(), io::Error> {
    match fs::rename(source_file_path, target_file_path) {
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
        let file_path = "test-dir";
        fs::create_dir(file_path).expect("Failed to create file");
        assert!(delete_file(file_path).is_err());
        fs::remove_dir(file_path).expect("Failed to remove file");
    }

    #[test]
    fn test_delete_file_does_not_exist() {
        let file_path = "test.txt";
        assert!(delete_file(file_path).is_err());
    }

    #[test]
    fn test_move_file_reg() {
        let source_file_path = "source.txt";
        let target_file_path = "target.txt";
        fs::write(source_file_path, "").expect("Failed to create file");
        assert!(move_file(source_file_path, target_file_path).is_ok());
        assert!(fs::metadata(source_file_path).is_err());
        assert!(fs::metadata(target_file_path).is_ok());
        fs::remove_file(target_file_path).expect("Failed to remove file");
    }

    #[test]
    fn test_move_file_dir() {
        let source_file_path = "source-dir";
        let target_file_path = "target-dir";
        fs::create_dir(source_file_path).expect("Failed to create file");
        assert!(move_file(source_file_path, target_file_path).is_ok());
        assert!(fs::metadata(source_file_path).is_err());
        assert!(fs::metadata(target_file_path).is_ok());
        fs::remove_dir_all(target_file_path).expect("Failed to remove file");
    }
}
