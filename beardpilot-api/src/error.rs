#[derive(Debug, thiserror::Error)]
pub enum EndpointError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Failed to deserialize version response: {0}")]
    DeserializationError(#[from] serde_json::Error),
    #[error("Ollama error: {0}")]
    ClientError(String),
    #[error("Parser error {0}")]
    ParserError(#[from] url::ParseError),
}

#[derive(Debug, serde::Deserialize)]
pub struct ProviderError {
    pub error: String,
}
