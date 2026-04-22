use crossterm::event::Event;
use ollama_minapi::endpoint::chat::{Chat, ChatResponse};
use tokio::sync::{mpsc, oneshot};

use crate::error::AppError;

pub enum AppEvent {
    UiEvent(Event),
    ResponseChunk(ChatResponse),
    SubmitResponse(Result<String, AppError>),
}

pub enum SessionEvent {
    SendChat(Chat),
    ConfirmationRequest {
        prompt: String,
        response: oneshot::Sender<bool>,
    },
}

pub enum UiAction {
    Submit(String), // user pressed Enter with this text
    Quit,           // Ctrl+C / Ctrl+Q
    None,           // event consumed, nothing for App to do
}
