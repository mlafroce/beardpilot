use beardpilot_api::endpoint::chat::{Chat, ChatStreamResponse};
use crossterm::event::Event;

pub enum AppEvent {
    UiEvent(Event),
    ResponseChunk(ChatStreamResponse),
}

pub enum SessionEvent {
    SendChat(Chat),
}

pub enum UiAction {
    Submit(String), // user pressed Enter with this text
    Quit,           // Ctrl+C / Ctrl+Q
    None,           // event consumed, nothing for App to do
}
