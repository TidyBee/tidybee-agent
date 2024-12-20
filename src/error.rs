use config::ConfigError as config_error;
use std::io::Error as io_error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error(transparent)]
    InvalidConfig(#[from] config_error),
    #[error(transparent)]
    Io(#[from] io_error),
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
    #[error("gRPC client is not connected")]
    ClientNotConnected(),
    #[error("Error creating file info")]
    FileInfoError(),
    #[error("Error sending event to Hub")]
    EventSendError(),
}
