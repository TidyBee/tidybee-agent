use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Path entry isn't a directory")]
    NotADirectory(),
}
