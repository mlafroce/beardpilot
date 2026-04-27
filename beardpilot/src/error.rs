use beardpilot_api::error::EndpointError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Client error: {0}")]
    Client(#[from] EndpointError),
}
