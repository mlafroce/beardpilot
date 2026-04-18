use ollama_rs::generation::chat::{ChatMessage, ChatMessageResponse};

#[derive(Clone, Default, PartialEq)]
pub enum ResponseStatus {
    #[default]
    Waiting,
    Thinking,
    ReceiveResponse,
}

/// Metadata about the model being used in this conversation.
#[derive(Clone, Default)]
pub struct ModelInfo {
    pub model_name: String,
    pub max_tokens: Option<usize>,
}

impl ModelInfo {
    pub fn new(model_name: impl Into<String>, max_tokens: Option<usize>) -> Self {
        Self {
            model_name: model_name.into(),
            max_tokens,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Role {
    User,
    Assistant,
    Thinking,
    Info,
    Error,
}

#[derive(Clone)]
pub struct Message {
    pub role: Role,
    pub text: String,
}

impl Message {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            text: text.into(),
        }
    }
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            text: text.into(),
        }
    }
    pub fn thinking(text: impl Into<String>) -> Self {
        Self {
            role: Role::Thinking,
            text: text.into(),
        }
    }
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            role: Role::Info,
            text: text.into(),
        }
    }
    pub fn error(text: impl Into<String>) -> Self {
        Self {
            role: Role::Error,
            text: text.into(),
        }
    }
}

/// Owns the chat history and injects a system prompt when present.
#[derive(Clone, Default)]
pub struct Conversation {
    messages: Vec<Message>,
    system_prompt: Option<String>,
    model_info: ModelInfo,
    response_status: ResponseStatus,
}

impl Conversation {
    pub fn new(system_prompt: Option<String>, model_info: ModelInfo) -> Self {
        Self {
            messages: vec![],
            system_prompt,
            model_info,
            response_status: ResponseStatus::Waiting,
        }
    }

    /// Return a reference to the model metadata.
    pub fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }

    pub fn push_user(&mut self, text: String) {
        let msg = Message::user(text);
        self.messages.push(msg);
    }

    /// Add a response chunk
    pub fn push_chunk(&mut self, response: ChatMessageResponse) {
        if let Some(thinking) = &response.message.thinking {
            if self.response_status != ResponseStatus::Thinking {
                let msg = Message::thinking("");
                self.messages.push(msg);
            }
            let message = self.messages.last_mut().unwrap();
            message.text.push_str(&thinking);
            self.response_status = ResponseStatus::Thinking;
        } else {
            if self.response_status != ResponseStatus::ReceiveResponse {
                let msg = Message::assistant("");
                self.messages.push(msg);
            }
            let message = self.messages.last_mut().unwrap();
            message.text.push_str(&response.message.content);
            self.response_status = ResponseStatus::ReceiveResponse;
        }
        if let Some(data) = &response.final_data {
            self.messages.push(Message::info(format!(
                "Tokens sent: {} | received: {}",
                data.prompt_eval_count, data.eval_count
            )));
        }
        if response.done {
            self.response_status = ResponseStatus::Waiting;
        }
    }

    /// Return the full message list
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// 
    pub fn response_status(&self) -> ResponseStatus {
        self.response_status.clone()
    }

    /// Return the filtered message list to be sent to the model,
    /// prepending the system prompt when configured.
    pub fn client_messages(&self) -> Vec<ChatMessage> {
        let mut all: Vec<ChatMessage> = Vec::with_capacity(self.messages.len() + 1);
        if let Some(ref prompt) = self.system_prompt {
            all.push(ChatMessage::system(prompt.clone()));
        }
        all.extend(self.messages.iter().filter_map(|m| match m.role {
            Role::User => Some(ChatMessage::user(m.text.clone())),
            Role::Assistant => Some(ChatMessage::assistant(m.text.clone())),
            _ => None,
        }));
        all
    }

    /// Wipe all history (system prompt is preserved).
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Remove the last user message (used to roll back on error).
    pub fn pop_last_user(&mut self) {
        if let Some(pos) = self.messages.iter().rposition(|m| m.role == Role::User) {
            self.messages.remove(pos);
        }
    }
}
