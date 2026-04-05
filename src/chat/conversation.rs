use ollama_rs::generation::chat::ChatMessage;

/// Metadata about the model being used in this conversation.
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


#[derive(Clone)]
pub enum Role {
    User,
    Assistant,
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
    messages: Vec<ChatMessage>,
    system_prompt: Option<String>,
    max_history: Option<usize>,
    model_info: ModelInfo,
}

impl Conversation {
    pub fn new(
        system_prompt: Option<String>,
        max_history: Option<usize>,
        model_info: ModelInfo,
    ) -> Self {
        Self {
            messages: vec![],
            system_prompt,
            max_history,
            model_info,
        }
    }

    /// Return a reference to the model metadata.
    pub fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }

    /// Append a user message and trim history if needed.
    pub fn add_user(&mut self, input: String) {
        self.messages.push(ChatMessage::user(input));
        self.trim();
    }

    /// Append an assistant reply.
    pub fn add_assistant(&mut self, response: String) {
        self.messages.push(ChatMessage::assistant(response));
    }

    /// Return the full message list to be sent to the model,
    /// prepending the system prompt when configured.
    pub fn messages(&self) -> Vec<ChatMessage> {
        let mut all: Vec<ChatMessage> = Vec::with_capacity(self.messages.len() + 1);
        if let Some(ref prompt) = self.system_prompt {
            all.push(ChatMessage::system(prompt.clone()));
        }
        all.extend(self.messages.iter().cloned());
        all
    }

    /// Wipe all history (system prompt is preserved).
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Remove the last user message (used to roll back on error).
    pub fn pop_last_user(&mut self) {
        if let Some(pos) = self
            .messages
            .iter()
            .rposition(|m| m.role == ollama_rs::generation::chat::MessageRole::User)
        {
            self.messages.remove(pos);
        }
    }

    /// Drop the oldest messages when over the limit.
    fn trim(&mut self) {
        if let Some(max) = self.max_history {
            while self.messages.len() > max {
                self.messages.remove(0);
            }
        }
    }
}
