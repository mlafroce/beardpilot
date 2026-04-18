use crossterm::event::Event;
use ollama_rs::generation::chat::ChatMessageResponse;
use tokio::sync::{mpsc, oneshot};

use crate::chat::conversation::Conversation;
use crate::error::AppError;

pub enum AppEvent {
    UiEvent(Event),
    ResponseChunk(ChatMessageResponse),
    SubmitResponse(Result<String, AppError>),
}

pub enum SessionEvent {
    SubmitPrompt(Conversation, mpsc::UnboundedSender<AppEvent>),
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
