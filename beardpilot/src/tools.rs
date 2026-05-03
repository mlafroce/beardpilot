pub mod bash;
pub mod find;
pub mod list_files;
pub mod read;

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}
