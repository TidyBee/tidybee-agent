use std::io::Error as io_error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error")]
    Io(#[from] io_error),
    #[error("Path entry isn't a directory")]
    NotDirectory(),
}
