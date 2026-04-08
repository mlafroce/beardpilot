use crossterm::event::Event;
use ollama_rs::generation::chat::ChatMessageResponse;
use tokio::sync::{oneshot, mpsc};

use crate::chat::conversation::Message;
use crate::error::AppError;

pub enum AppEvent {
    UiEvent(Event),
    MessageAdded(Message),
    SubmitResponse(Result<ChatMessageResponse, AppError>),
}

pub enum SessionEvent {
    SubmitPrompt(String, mpsc::UnboundedSender<AppEvent>),
    ConfirmationRequest{prompt: String, response: oneshot::Sender<bool>},
}

pub enum UiAction {
    Submit(String), // user pressed Enter with this text
    Quit,           // Ctrl+C / Ctrl+Q
    Redraw, // resize or cosmetic key
    None,   // event consumed, nothing for App to do
}
