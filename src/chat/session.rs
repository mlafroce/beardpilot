use crate::chat::conversation::{Conversation, ModelInfo};
use crate::config::AppConfig;
use crate::error::AppError;
use crate::event::{AppEvent, SessionEvent};
use futures_util::StreamExt;
use ollama_rs::Ollama;
use ollama_rs::generation::chat::request::ChatMessageRequest;
use tokio::sync::mpsc::{self, unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::debug;

pub struct Session {
    ollama: Ollama,
    model: String,
    sender: UnboundedSender<SessionEvent>,
    receiver: UnboundedReceiver<SessionEvent>,
}

impl Session {
    pub fn new(config: &AppConfig, ollama: Ollama) -> Result<Self, AppError> {
        let (sender, receiver) = unbounded_channel();
        let model_info = ModelInfo::new(&config.model, config.max_tokens);
        /*
        let coordinator = Coordinator::new(ollama, config.model.clone(), history)
            .add_tool(Find::new(sender.clone()))
            .add_tool(ListFiles::default())
            .add_tool(Read::default())
            .add_tool(Bash::default())
            .debug(true);
         */
        Ok(Self {
            ollama,
            model: config.model.clone(),
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
                SessionEvent::SubmitPrompt(conversation, resp_sender) => {
                    debug!("SessionEvent::SubmitPrompt");
                    let resp = self.send_chat(conversation, &resp_sender).await;
                    let _ = resp_sender.send(AppEvent::SubmitResponse(resp));
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
        conversation: Conversation,
        app_sender: &mpsc::UnboundedSender<AppEvent>,
    ) -> Result<String, AppError> {
        // Handle slash-commands
        /*
        match text.as_str() {
            "exit" | "/exit" => {
                self.messages.push(Message::info("Goodbye!"));
                return Ok(());
            }
            "/clear" | "/reset" => {
                self.conversation.clear();
                self.messages.clear();
                self.messages.push(Message::info("Conversation history cleared."));
                return Ok(());
            }
            "/help" | "/h" | "help" => {
                self.messages.push(Message::info(concat!(
                    "Commands:\n",
                    "  /clear, /reset  — clear conversation history\n",
                    "  /help, /h       — show this help\n",
                    "  Ctrl+C          — quit\n",
                    "  ↑/↓, PgUp/PgDn — scroll messages\n",
                    "  ←/→             — move cursor\n",
                    "  Home/End        — jump to start/end of input\n",
                    "  Click           — position cursor with mouse\n",
                    "  Scroll wheel    — scroll messages",
                )));
                return Ok(());
            }
            _ => {}
        }
        */

        debug!("SessionEvent::submit, calling ollama");
        let request = ChatMessageRequest::new(self.model.clone(), conversation.client_messages());
        let mut response = self.ollama.send_chat_messages_stream(request).await?;
        let mut final_response = String::new();
        while let Some(chunk) = response.next().await {
            let _ = app_sender.send(AppEvent::ResponseChunk(
                chunk.clone().expect("Invalid chunk"),
            ));
            final_response.push_str(&chunk.unwrap().message.content);
        }
        //self.handle_submit_response(&response, app_sender);
        Ok(final_response)
    }
}
