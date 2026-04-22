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
    pub format: String,

    /// Runtime options that control text generation
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

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Message {
    /// Author of the message.
    pub role: MessageRole,

    /// Message text content
    #[serde(default)]
    pub content: String,

    /// Optional deliberate thinking trace when `think` is enabled
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub thinking: String,

    /// List of inline images for multimodal models
    /// Base64-encoded image content
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<String>,

    /// Tool call requests produced by the model
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
}

impl Message {
    pub fn system(content: String) -> Self {
        Self {role: MessageRole::System, content, thinking: String::new(), images: vec![], tool_calls: vec![]}
    }

    pub fn user(content: String) -> Self {
        Self {role: MessageRole::User, content, thinking: String::new(), images: vec![], tool_calls: vec![]}
    }

    pub fn assistant(content: String) -> Self {
        Self {role: MessageRole::Assistant, content, thinking: String::new(), images: vec![], tool_calls: vec![]}
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ToolCall {
    id: String,
    function: ToolCallFunction
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ToolCallFunction {
    index: Value,
    name: String,
    arguments: HashMap<String, Value>
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChatResponse {
    ///Model name
    pub model: String,

    ///ISO 8601 timestamp of response creation
    pub created_at: String,

    ///The model's generated text response
    pub message: Message,

    ///Indicates whether generation has finished
    pub done: bool,

    ///Reason the generation stopped
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done_reason: Option<String>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub final_data: Option<ChatMessageFinalResponseData>,
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
