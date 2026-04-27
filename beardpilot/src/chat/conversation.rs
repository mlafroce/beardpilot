use beardpilot_api::endpoint::{
    chat::{Chat, ChatResponse, Message},
    tool::{tool_to_json, Tool},
};
use tracing::debug;

use crate::tools::list_files::ListFiles;

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
pub struct ConversationMessage {
    pub role: Role,
    pub text: String,
}

impl ConversationMessage {
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
pub struct Conversation {
    messages: Vec<ConversationMessage>,
    system_prompt: Option<String>,
    pub model_info: ModelInfo,
    tools: Vec<Box<dyn Tool>>,
    response_status: ResponseStatus,
}

impl Conversation {
    pub fn new(system_prompt: Option<String>, model_info: ModelInfo) -> Self {
        Self {
            messages: vec![],
            tools: vec![Box::new(ListFiles {})],
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
        let msg = ConversationMessage::user(text);
        self.messages.push(msg);
    }

    /// Add a response chunk
    pub fn push_chunk(&mut self, response: ChatResponse) {
        debug!("Chunk received: {:?}", response);
        if !response.thinking().is_empty() {
            if self.response_status != ResponseStatus::Thinking {
                let msg = ConversationMessage::thinking("");
                self.messages.push(msg);
            }
            let message = self.messages.last_mut().unwrap();
            message.text.push_str(&response.choices[0].delta.thinking);
            self.response_status = ResponseStatus::Thinking;
        } else {
            if self.response_status != ResponseStatus::ReceiveResponse {
                let msg = ConversationMessage::assistant("");
                self.messages.push(msg);
            }
            let message = self.messages.last_mut().unwrap();
            message.text.push_str(&response.content());
            self.response_status = ResponseStatus::ReceiveResponse;
        }
        if let Some(data) = &response.final_data {
            self.messages.push(ConversationMessage::info(format!(
                "Tokens sent: {} | received: {}",
                data.prompt_eval_count, data.eval_count
            )));
        }
        if response.done().is_some() {
            self.response_status = ResponseStatus::Waiting;
        }
    }

    /// Return the full message list
    pub fn messages(&self) -> &[ConversationMessage] {
        &self.messages
    }

    ///
    pub fn response_status(&self) -> ResponseStatus {
        self.response_status.clone()
    }

    pub fn session_chat(&self) -> Chat {
        let messages = self.session_messages();
        let model = self.model_info.model_name.clone();
        let tools = self
            .tools
            .iter()
            .map(|t| tool_to_json(t.as_ref()))
            .collect();
        Chat {
            model,
            messages,
            tools,
            ..Default::default()
        }
    }

    fn session_messages(&self) -> Vec<Message> {
        let mut all: Vec<Message> = Vec::with_capacity(self.messages.len() + 1);
        if let Some(ref prompt) = self.system_prompt {
            all.push(Message::system(&prompt));
        }
        all.extend(self.messages.iter().filter_map(|m| match m.role {
            Role::User => Some(Message::user(&m.text)),
            Role::Assistant => Some(Message::assistant(&m.text)),
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
