use futures_util::Stream;
use log::debug;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::error::EndpointError;

pub mod chat;
pub mod embed;
pub mod generate;
pub mod model;
pub mod tag;
pub mod tool;
pub mod version;

/// A wrapper stream that deserializes JSON responses and handles errors
pub struct EndpointStream<S, ResponseItem> {
    pub(crate) stream: S,
    pub(crate) _data: PhantomData<ResponseItem>,
    pub(crate) buffer: String,
}

//impl<S, B, ResponseItem> Stream for EndpointStream<S, ResponseItem>
impl<S, B, ResponseItem> Stream for EndpointStream<S, ResponseItem>
where
    S: Stream<Item = Result<B, reqwest::Error>> + Unpin,
    B: AsRef<[u8]>,
    ResponseItem: serde::de::DeserializeOwned + Unpin + Debug,
{
    type Item = Result<ResponseItem, EndpointError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        debug!("Polling endpoint...");
        let tmp = match Pin::new(&mut self.stream).poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                debug!("Poll::Ready {:?}", str::from_utf8(bytes.as_ref()));
                self.handle_stream_bytes(bytes).map_or_else(
                    || {
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    },
                    |item| Poll::Ready(Some(item)),
                )
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(EndpointError::NetworkError(e)))),
            Poll::Ready(None) => {
                debug!("Poll::Ready(None)");
                self.pull_from_buffer()
                    .map_or(Poll::Ready(None), |item| Poll::Ready(Some(item)))
            }
            Poll::Pending => Poll::Pending,
        };
        debug!("Poll returned {:?} ", tmp);
        tmp
    }
}

impl<S, B, ResponseItem> EndpointStream<S, ResponseItem>
where
    S: Stream<Item = Result<B, reqwest::Error>> + Unpin,
    B: AsRef<[u8]>,
    ResponseItem: serde::de::DeserializeOwned + Unpin,
{
    fn handle_stream_bytes(&mut self, bytes: B) -> Option<Result<ResponseItem, EndpointError>> {
        let s = match str::from_utf8(bytes.as_ref()) {
            Ok(s) => s,
            Err(e) => return Some(Err(EndpointError::ClientError(e.to_string()))),
        };
        debug!("Pushing to buffer: {:?}", s);
        // Add to buffer
        self.buffer.push_str(s);
        self.pull_from_buffer()
    }

    fn pull_from_buffer(&mut self) -> Option<Result<ResponseItem, EndpointError>> {
        // Split by double newlines to find message boundaries
        let json_chunk = if let Some(pos) = self.buffer.find("\n\n") {
            let chunk: String = self.buffer.drain(0..pos).collect();
            // Also remove the "\n\n" delimiter from the buffer
            self.buffer.drain(0.."\n\n".len());
            chunk
        } else {
            self.buffer.drain(..).collect()
        };
        let trimmed = if let Some(stripped) = json_chunk.strip_prefix("data: ") {
            stripped.trim().to_string()
        } else {
            json_chunk.trim().to_string()
        };
        if !trimmed.is_empty() {
            debug!("Deserializing: {:?}", trimmed);
            match serde_json::from_str::<ResponseItem>(&trimmed) {
                Ok(response) => return Some(Ok(response)),
                Err(e) => return Some(Err(EndpointError::DeserializationError(e))),
            }
        }
        // No complete message yet, wait for more data
        None
    }
}
