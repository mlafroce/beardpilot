use ollama_rs::error::OllamaError;
use ollama_rs::generation::chat::ChatMessageResponse;
use ollama_rs::{coordinator::Coordinator, generation::chat::ChatMessage, Ollama};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender, unbounded_channel};
use tracing::debug;
use crate::chat::conversation::{Conversation, Message, ModelInfo};
use crate::config::AppConfig;
use crate::error::AppError;
use crate::event::{AppEvent, SessionEvent};
use crate::tools::{bash::Bash, find::Find, list_files::ListFiles, read::Read};

pub struct Session {
    pub messages: Vec<Message>,
    pub conversation: Conversation,
    coordinator: Coordinator<Vec<ChatMessage>>,
    sender: UnboundedSender<SessionEvent>,
    receiver: UnboundedReceiver<SessionEvent>
}

impl Session {
    pub fn new(config: &AppConfig, ollama: Ollama) -> Result<Self, AppError> {
        let (sender, receiver) = unbounded_channel();
        let history = vec![];
        let model_info = ModelInfo::new(&config.model, config.max_tokens);
        let coordinator = Coordinator::new(ollama, config.model.clone(), history)
            .add_tool(Find::new(sender.clone()))
            .add_tool(ListFiles::default())
            .add_tool(Read::default())
            .add_tool(Bash::default());
        let conversation = Conversation::new(config.system_prompt.clone(), config.max_history, model_info);
        Ok(Self {
            messages: vec![],
            conversation,
            coordinator,
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
                SessionEvent::SubmitPrompt(text, resp_sender) => {
                    debug!("SessionEvent::SubmitPrompt");
                    let resp = self.submit(text, &resp_sender).await;
                    let _ = resp_sender.send(AppEvent::SubmitResponse(resp));
                }
                SessionEvent::ConfirmationRequest { prompt, response } => {
                    debug!("Confirmation request: {}", prompt);
                    let _ = response.send(true);
                }
            }
        }
    }

    pub async fn submit(&mut self, text: String, app_sender: &mpsc::UnboundedSender<AppEvent>) -> Result<ChatMessageResponse, AppError> {
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

        let user_msg = Message::user(&text);
        self.push_msg(user_msg, app_sender);
        self.conversation.add_user(text);

        debug!("SessionEvent::submit, calling coordinator");
        let response = self.coordinator.chat(self.conversation.messages()).await;
        self.handle_submit_response(&response, app_sender);
        Ok(response?)
    }

    fn push_msg(&mut self, msg: Message, app_sender: &mpsc::UnboundedSender<AppEvent>) {
        let _ = app_sender.send(AppEvent::MessageAdded(msg.clone()));
        self.messages.push(msg);
    }

    pub fn handle_submit_response(&mut self, response: &Result<ChatMessageResponse, OllamaError>, app_sender: &mpsc::UnboundedSender<AppEvent>) {
        match response {
            Ok(res) => {
                let mut content = String::new();
                if let Some(thinking) = &res.message.thinking {
                    if !thinking.is_empty() {
                        self.push_msg(Message::info(format!("(thinking)\n{}", thinking)), app_sender);
                    }
                }
                content += &res.message.content;
                self.push_msg(Message::assistant(&content), app_sender);
                self.conversation.add_assistant(content);

                if let Some(data) = &res.final_data {
                    self.push_msg(Message::info(format!(
                        "Tokens sent: {} | received: {}",
                        data.prompt_eval_count, data.eval_count
                    )), app_sender);
                }
            }
            Err(e) => {
                self.conversation.pop_last_user();
                self.push_msg(Message::error(e.to_string()), app_sender);
            }
        }
    }
}