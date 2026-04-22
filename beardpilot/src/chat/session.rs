use crate::error::AppError;
use crate::event::{AppEvent, SessionEvent};
use futures_util::StreamExt;
use ollama_minapi::endpoint::chat::Chat;
use ollama_minapi::Ollama;
use tokio::sync::mpsc::{self, unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::debug;

pub struct Session {
    ollama: Ollama,
    app_sender: mpsc::UnboundedSender<AppEvent>,
    sender: UnboundedSender<SessionEvent>,
    receiver: UnboundedReceiver<SessionEvent>,
}

impl Session {
    pub fn new(ollama: Ollama, app_sender: mpsc::UnboundedSender<AppEvent>) -> Result<Self, AppError> {
        let (sender, receiver) = unbounded_channel();
        Ok(Self {
            ollama,
            app_sender,
            sender,
            receiver,
        })
    }

    pub fn get_sender(&self) -> UnboundedSender<SessionEvent> {
        self.sender.clone()
    }

    pub async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            match message {
                SessionEvent::SendChat(messages) => {
                    debug!("SessionEvent::SubmitPrompt");
                    let resp = self.send_chat(messages).await;
                    let _ = self.app_sender.send(AppEvent::SubmitResponse(resp));
                }
                SessionEvent::ConfirmationRequest { prompt, response } => {
                    debug!("Confirmation request: {}", prompt);
                    let _ = response.send(true);
                }
            }
        }
    }

    pub async fn send_chat(
        &mut self,
        chat: Chat,
    ) -> Result<String, AppError> {
        debug!("SessionEvent::submit, calling ollama");
        let mut response = self.ollama.post_chat_stream(chat).await?;
        let mut final_response = String::new();
        while let Some(chunk) = response.next().await {
            let chunk = chunk?;
            let _ = self.app_sender.send(AppEvent::ResponseChunk(chunk.clone()));
            final_response.push_str(&chunk.message.content);
        }
        Ok(final_response)
    }
}
