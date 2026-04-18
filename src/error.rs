use ollama_rs::error::OllamaError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Ollama error: {0}")]
    Ollama(#[from] OllamaError),
}
