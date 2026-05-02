use std::collections::HashMap;

use beardpilot_api::endpoint::chat::{
    Chat, ChatStreamResponse, FinishReason, Message, ToolCallMessage,
};

use crate::{
    chat::tool_registry::{ToolCall, ToolRegistry},
    error::AppResult,
};

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
pub enum LocalMessage {
    User(String),
    Assistant(String),
    ToolCall(ToolCall),
    ToolResponse { id: String, response: String },
    Thinking(String),
    Info(String),
}

/// Owns the chat history and injects a system prompt when present.
pub struct Conversation {
    messages: Vec<LocalMessage>,
    system_prompt: Option<String>,
    tool_registry: ToolRegistry,
    pub model_info: ModelInfo,
    conversation_status: ResponseStatus,
    tool_call_buffer: ToolCallBuffer,
}

impl Conversation {
    pub fn new(system_prompt: Option<String>, model_info: ModelInfo) -> Self {
        Self {
            messages: vec![],
            tool_registry: ToolRegistry::new(),
            system_prompt,
            model_info,
            conversation_status: ResponseStatus::Waiting,
            tool_call_buffer: ToolCallBuffer::default(),
        }
    }

    /// Return a reference to the model metadata.
    pub fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }

    pub fn push_user(&mut self, text: String) {
        let msg = LocalMessage::User(text);
        self.messages.push(msg);
    }

    /// Append a streaming response chunk to the conversation.
    pub async fn push_chunk(&mut self, response: ChatStreamResponse) -> AppResult<()> {
        self.tool_call_buffer.push(response.tool_calls());
        self.append_stream_text(&response);
        match response.done() {
            Some(FinishReason::ToolCalls) => self.flush_and_execute_tool_calls().await?,
            _ => self.append_token_usage(&response),
        }
        Ok(())
    }

    /// Append the thinking or content text from this chunk to the last message,
    /// creating a new message when the response phase changes.
    fn append_stream_text(&mut self, response: &ChatStreamResponse) {
        let thinking = response.thinking();
        if !thinking.is_empty() {
            if let Some(LocalMessage::Thinking(text)) = self.messages.last_mut() {
                text.push_str(thinking);
            } else {
                self.conversation_status = ResponseStatus::Thinking;
                self.messages
                    .push(LocalMessage::Thinking(thinking.to_owned()));
            }
        } else {
            if let Some(LocalMessage::Assistant(text)) = self.messages.last_mut() {
                text.push_str(response.content());
            } else {
                self.conversation_status = ResponseStatus::ReceiveResponse;
                self.messages
                    .push(LocalMessage::Assistant(response.content().to_owned()));
            }
        }
    }

    /// If final token-usage data is present, append an info message.
    fn append_token_usage(&mut self, response: &ChatStreamResponse) {
        if let Some(data) = &response.final_data {
            self.messages.push(LocalMessage::Info(format!(
                "Tokens sent: {} | received: {}",
                data.prompt_eval_count, data.eval_count
            )));
        }
    }

    /// Drain the tool-call buffer, execute each call, and append the results.
    async fn flush_and_execute_tool_calls(&mut self) -> AppResult<()> {
        let tool_calls = self.tool_call_buffer.take();
        for call in tool_calls {
            let id = call.id.clone();
            self.messages.push(LocalMessage::ToolCall(call.clone()));
            let response = self.tool_registry.call_tool(call).await?;
            self.messages
                .push(LocalMessage::ToolResponse { id, response });
        }
        self.conversation_status = ResponseStatus::Waiting;
        Ok(())
    }

    /// Return the full message list
    pub fn messages(&self) -> &[LocalMessage] {
        &self.messages
    }

    pub fn conversation_status(&self) -> ResponseStatus {
        self.conversation_status.clone()
    }

    pub fn session_chat(&self) -> Chat {
        let messages = self.session_messages();
        let model = self.model_info.model_name.clone();
        let tools = self.tool_registry.get_chat_tools();
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
            all.push(Message::system(prompt));
        }
        all.extend(self.messages.iter().filter_map(|m| match m {
            LocalMessage::User(content) => Some(Message::user(content)),
            LocalMessage::Assistant(content) => Some(Message::assistant(content)),
            LocalMessage::ToolCall(call) => {
                Some(Message::tool_calls(vec![call.to_tool_call_message()]))
            }
            LocalMessage::ToolResponse { id, response } => {
                Some(Message::tool_response(id.clone(), response.clone()))
            }
            _ => None,
        }));
        all
    }
}

/// Helper class to handle stream tool calls
#[derive(Default)]
struct ToolCallBuffer {
    tool_calls: HashMap<u32, ToolCallData>,
}

struct ToolCallData {
    id: String,
    name: String,
    arguments: String,
}

impl ToolCallBuffer {
    pub fn push(&mut self, deltas: Option<Vec<ToolCallMessage>>) {
        for delta in deltas.iter().flatten() {
            let idx = delta.function.index.unwrap_or(0);
            self.tool_calls
                .entry(idx)
                .and_modify(|tcd| {
                    tcd.id += &delta.id;
                    tcd.name += &delta.function.name;
                    tcd.arguments += &delta.function.arguments;
                })
                .or_insert(ToolCallData {
                    id: delta.id.clone(),
                    name: delta.function.name.clone(),
                    arguments: delta.function.arguments.clone(),
                });
        }
    }

    pub fn take(&mut self) -> Vec<ToolCall> {
        self.tool_calls
            .drain()
            .map(
                |(
                    _,
                    ToolCallData {
                        id,
                        name,
                        arguments,
                    },
                )| {
                    let arguments = serde_json::from_str(&arguments).unwrap();
                    ToolCall {
                        id,
                        function: name,
                        arguments,
                    }
                },
            )
            .collect()
    }
}
