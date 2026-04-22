use crate::endpoint::chat::ChatMessageFinalResponseData;

/// "Generate" endpoint body
#[derive(Debug, Default, serde::Serialize)]
pub struct Generate {
    /// Model name
    pub model: String,

    /// Text for the model to generate a response from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Used for fill-in-the-middle models, text that appears after the user prompt and before the model response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,

    /// Base64-encoded images for models that support image input
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,

    /// Structured output format for the model to generate a response from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// System prompt for the model to generate a response from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,

    /// When true, returns a stream of partial responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// When true, returns separate thinking output in addition to content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub think: Option<bool>,

    /// When true, returns the raw response from the model without any prompt templating
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<bool>,

    /// Model keep-alive duration (for example 5m or 0 to unload immediately)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GenerateResponse {
    ///Model name
    pub model: String,

    ///ISO 8601 timestamp of response creation
    pub created_at: String,

    ///The model's generated text response
    pub response: String,

    ///The model's generated thinking output
    pub thinking: Option<String>,

    ///Indicates whether generation has finished
    pub done: bool,

    ///Reason the generation stopped
    pub done_reason: Option<String>,
    #[serde(flatten)]
    pub final_data: Option<ChatMessageFinalResponseData>,
}
