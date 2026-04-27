use std::collections::HashMap;

use serde_json::Value;

/// Generate the next chat message in a conversation between a user and an assistant.
#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct Chat {
    /// Model name
    pub model: String,

    /// Chat history as an array of message objects (each with a role and content)
    pub messages: Vec<Message>,

    /// Optional list of function tools the model may call during the chat
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<serde_json::Value>,

    /// Format to return a response in. Can be `json` or a JSON schema
    #[serde(skip_serializing_if = "String::is_empty")]
    pub format: String,

    /// Runtime options that control text generation
    #[serde(skip_serializing_if = "CallOptions::is_empty")]
    pub options: CallOptions,

    pub stream: bool,
    /// When true, returns separate thinking output in addition to content.
    ///  Can be a boolean (true/false) or a String ("high", "medium", "low") for supported models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub think: Option<bool>,

    /// Model keep-alive duration (for example 5m or 0 to unload immediately)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<String>,

    /// Whether to return log probabilities of the output tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// Number of most likely tokens to return at each token position when logprobs are enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<i64>,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct CallOptions {
    /// Random seed used for reproducible outputs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    /// Controls randomness in generation (higher = more random)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Limits next token selection to the K most likely
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i64>,

    /// Cumulative probability threshold for nucleus sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    /// Minimum probability threshold for token selection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_p: Option<f64>,

    /// Stop sequences that will halt generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<String>,

    /// Context length size (number of tokens)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<i64>,

    /// Maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<i64>,
}

impl CallOptions {
    pub fn is_empty(&self) -> bool {
        self.min_p.is_none()
            && self.num_ctx.is_none()
            && self.num_predict.is_none()
            && self.seed.is_none()
            && self.stop.is_none()
            && self.temperature.is_none()
            && self.top_k.is_none()
            && self.top_p.is_none()
    }
}

/// Builder for creating Chat instances with a fluent interface
pub struct ChatBuilder {
    model: String,
    messages: Vec<Message>,
    tools: Vec<serde_json::Value>,
    format: String,
    options: CallOptions,
    stream: bool,
    think: Option<bool>,
    keep_alive: Option<String>,
    logprobs: Option<bool>,
    top_logprobs: Option<i64>,
}

impl Chat {
    /// Create a new ChatBuilder with the required fields
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> ChatBuilder {
        ChatBuilder {
            model: model.into(),
            messages,
            tools: vec![],
            format: String::new(),
            options: CallOptions::default(),
            stream: false,
            think: None,
            keep_alive: None,
            logprobs: None,
            top_logprobs: None,
        }
    }
}

impl ChatBuilder {
    /// Set the tools for the chat
    pub fn with_tools(mut self, tools: Vec<serde_json::Value>) -> Self {
        self.tools = tools;
        self
    }

    /// Set the format for the chat response
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = format.into();
        self
    }

    /// Set the call options for the chat
    pub fn with_options(mut self, options: CallOptions) -> Self {
        self.options = options;
        self
    }

    /// Set whether to stream the response
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    /// Set the think option
    pub fn with_think(mut self, think: bool) -> Self {
        self.think = Some(think);
        self
    }

    /// Set the keep_alive duration
    pub fn with_keep_alive(mut self, keep_alive: impl Into<String>) -> Self {
        self.keep_alive = Some(keep_alive.into());
        self
    }

    /// Set whether to return log probabilities
    pub fn with_logprobs(mut self, logprobs: bool) -> Self {
        self.logprobs = Some(logprobs);
        self
    }

    /// Set the number of top log probabilities to return
    pub fn with_top_logprobs(mut self, top_logprobs: i64) -> Self {
        self.top_logprobs = Some(top_logprobs);
        self
    }

    /// Build the Chat instance
    pub fn build(self) -> Chat {
        Chat {
            model: self.model,
            messages: self.messages,
            tools: self.tools,
            format: self.format,
            options: self.options,
            stream: self.stream,
            think: self.think,
            keep_alive: self.keep_alive,
            logprobs: self.logprobs,
            top_logprobs: self.top_logprobs,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Message {
    /// Author of the message.
    /// In streams might not be present after first chunk.
    pub role: Option<MessageRole>,

    /// Message text content
    #[serde(default)]
    pub content: String,

    /// List of inline images for multimodal models
    /// Base64-encoded image content
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<String>,

    /// Tool call requests produced by the model
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
}

impl Message {
    pub fn system(content: &str) -> Self {
        Self {
            role: Some(MessageRole::System),
            content: content.to_owned(),
            images: vec![],
            tool_calls: vec![],
        }
    }

    pub fn user(content: &str) -> Self {
        Self {
            role: Some(MessageRole::User),
            content: content.to_owned(),
            images: vec![],
            tool_calls: vec![],
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: Some(MessageRole::Assistant),
            content: content.to_owned(),
            images: vec![],
            tool_calls: vec![],
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ResponseMessage {
    /// Optional deliberate thinking trace when `think` is enabled
    #[serde(default)]
    pub thinking: String,

    #[serde(flatten)]
    pub message: Message,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ToolCall {
    id: String,
    function: ToolCallFunction,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ToolCallFunction {
    index: Value,
    name: String,
    arguments: HashMap<String, Value>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChatResponse {
    /// Unique identifier for the response
    #[serde(default)]
    pub id: String,

    /// Object type (e.g., "chat.completion.chunk")
    #[serde(default)]
    pub object: String,

    ///Model name
    pub model: String,

    ///ISO 8601 timestamp of response creation
    pub created: i64,

    ///The model's generated text response
    //pub message: Message,
    pub choices: Vec<MessageChunk>,

    ///Indicates whether generation has finished
    #[serde(default)]
    pub done: bool,

    ///Reason the generation stopped
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub done_reason: Option<String>,

    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub final_data: Option<ChatMessageFinalResponseData>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MessageChunk {
    pub index: i64,
    pub delta: ResponseMessage,
    ///Indicates whether generation has finished
    #[serde(default)]
    pub finish_reason: Option<FinishReason>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChatMessageFinalResponseData {
    ///Time spent generating the response in nanoseconds
    pub total_duration: i64,

    ///Time spent loading the model in nanoseconds
    pub load_duration: i64,

    ///Number of input tokens in the prompt
    pub prompt_eval_count: i64,

    ///Time spent evaluating the prompt in nanoseconds
    pub prompt_eval_duration: i64,

    ///Number of output tokens generated in the response
    pub eval_count: i64,

    ///Time spent generating tokens in nanoseconds
    pub eval_duration: i64,
}

impl ChatResponse {
    pub fn thinking(&self) -> &str {
        &self.choices[0].delta.thinking
    }

    pub fn content(&self) -> &str {
        &self.choices[0].delta.message.content
    }

    pub fn role(&self) -> &Option<MessageRole> {
        &self.choices[0].delta.message.role
    }

    pub fn done(&self) -> Option<FinishReason> {
        if self.done {
            Some(FinishReason::Stop)
        } else {
            self.choices[0].finish_reason.clone()
        }
    }
}
