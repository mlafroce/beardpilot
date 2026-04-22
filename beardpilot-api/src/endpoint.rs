use futures_util::Stream;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

pub mod chat;
pub mod embed;
pub mod generate;
pub mod model;
pub mod tag;
pub mod tool;
pub mod version;

#[derive(Debug, thiserror::Error)]
pub enum EndpointError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Failed to deserialize version response: {0}")]
    DeserializationError(#[from] serde_json::Error),
    #[error("Ollama error: {0}")]
    OllamaError(String),
    #[error("Parser error {0}")]
    ParserError(#[from] url::ParseError),
}

#[derive(Debug, serde::Deserialize)]
pub struct OllamaError {
    pub error: String,
}

/// A wrapper stream that deserializes JSON responses and handles errors
pub struct EndpointStream<S, ResponseItem> {
    pub(crate) stream: S,
    pub(crate) _data: PhantomData<ResponseItem>,
}

//impl<S, B, ResponseItem> Stream for EndpointStream<S, ResponseItem>
impl<S, B, ResponseItem> Stream for EndpointStream<S, ResponseItem>
where
    S: Stream<Item = Result<B, reqwest::Error>> + Unpin,
    B: AsRef<[u8]>,
    ResponseItem: serde::de::DeserializeOwned + Unpin,
{
    type Item = Result<ResponseItem, EndpointError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.stream).poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                match serde_json::from_slice::<ResponseItem>(bytes.as_ref()) {
                    Ok(response) => Poll::Ready(Some(Ok(response))),
                    Err(e) => Poll::Ready(Some(Err(EndpointError::DeserializationError(e)))),
                }
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(EndpointError::NetworkError(e)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
