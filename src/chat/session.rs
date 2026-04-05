use ollama_rs::{coordinator::Coordinator, generation::chat::ChatMessage, Ollama};
use crate::chat::conversation::{Conversation, Message, ModelInfo};
use crate::config::AppConfig;
use crate::error::AppError;
use crate::tools::{bash::Bash, find::Find, list_files::ListFiles, read::Read};

pub struct Session {
    pub messages: Vec<Message>,
    pub conversation: Conversation,
    coordinator: Coordinator<Vec<ChatMessage>>,
}

impl Session {
    pub fn new(config: &AppConfig) -> Self {
        let history = vec![];
        let model_info = ModelInfo::new(&config.model, config.max_tokens);
        let ollama = Ollama::new(config.host.to_string(), config.port);
        let coordinator = Coordinator::new(ollama, config.model.clone(), history)
            .add_tool(Find::default())
            .add_tool(ListFiles::default())
            .add_tool(Read::default())
            .add_tool(Bash::default());
        let conversation = Conversation::new(config.system_prompt.clone(), config.max_history, model_info);
        Self {
            messages: vec![],
            conversation,
            coordinator,
        }
    }

    pub async fn submit(&mut self, text: String) -> Result<(), AppError> {
        // Handle slash-commands
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


        // Regular chat message
        self.messages.push(Message::user(&text));
        self.conversation.add_user(text);
        //self.thinking = true;

        let result = self.coordinator.chat(self.conversation.messages()).await;
        //elf.thinking = false;

        match result {
            Ok(res) => {
                let mut response = String::new();
                if let Some(thinking) = res.message.thinking {
                    if !thinking.is_empty() {
                        self.messages.push(Message::info(&format!("(thinking)\n{}", thinking)));
                    }
                }
                response += &res.message.content;
                self.messages.push(Message::assistant(&response));
                self.conversation.add_assistant(response);

                if let Some(data) = res.final_data {
                    self.messages.push(Message::info(&format!(
                        "Tokens sent: {} | received: {}",
                        data.prompt_eval_count, data.eval_count
                    )));
                }
            }
            Err(e) => {
                self.conversation.pop_last_user();
                self.messages.push(Message::error(e.to_string()));
            }
        }

        Ok(())
    }
}