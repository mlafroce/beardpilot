use beardpilot_api::endpoint::chat::{Chat, ChatResponse};
use crossterm::event::Event;
use tokio::sync::oneshot;

pub enum AppEvent {
    UiEvent(Event),
    ResponseChunk(ChatResponse),
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
