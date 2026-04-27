use crate::error::AppError;
use crate::event::{AppEvent, SessionEvent};
use beardpilot_api::client::MistralClient;
use beardpilot_api::endpoint::chat::{Chat, MessageRole};
use futures_util::StreamExt;
use tokio::sync::mpsc::{self, unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::debug;

pub struct Session {
    provider: MistralClient,
    app_sender: mpsc::UnboundedSender<AppEvent>,
    sender: UnboundedSender<SessionEvent>,
    receiver: UnboundedReceiver<SessionEvent>,
    /// Some models send the role once and only send it when it changes
    last_role: MessageRole,
}

impl Session {
    pub fn new(
        provider: MistralClient,
        app_sender: mpsc::UnboundedSender<AppEvent>,
    ) -> Result<Self, AppError> {
        let (sender, receiver) = unbounded_channel();
        Ok(Self {
            provider,
            app_sender,
            sender,
            receiver,
            last_role: MessageRole::Assistant,
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
                }
                SessionEvent::ConfirmationRequest { prompt, response } => {
                    debug!("Confirmation request: {}", prompt);
                    let _ = response.send(true);
                }
            }
        }
    }

    pub async fn send_chat(&mut self, chat: Chat) -> Result<(), AppError> {
        let mut response = self.provider.chat_stream(chat).await?;
        while let Some(chunk) = response.next().await {
            let chunk = chunk?;
            if let Some(new_role) = &chunk.role() {
                self.last_role = new_role.clone();
            }
            let _ = self.app_sender.send(AppEvent::ResponseChunk(chunk));
        }
        Ok(())
    }
}
