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

#[derive(Error, Debug)]
pub enum HubError {
    #[error("Unexpected error from the Hub: {0}")]
    UnExpectedError(String),
    #[error("Hub client creation failed: {0}")]
    HubClientCreationFailed(String),
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
    #[error("Maximum number of attemps to connect to the Hub reached without success")]
    MaximumAttemptsReached(),
    #[error(transparent)]
    EventClientError(#[from] GrpcClientError),
}

#[derive(Error, Debug)]
pub enum GrpcClientError {
    #[error(transparent)]
    InvalidEndpoint(#[from] tonic::transport::Error),
    #[error("Agent UUID not set")]
    AgentUuidNotSet(),
}
