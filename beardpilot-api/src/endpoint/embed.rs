use crate::endpoint::chat::CallOptions;

/// "Embed" endpoint body
#[derive(Debug, Default, serde::Serialize)]
pub struct Embed {
    /// Model name
    pub model: String,

    /// Text or array of texts to generate embeddings for
    pub input: String,

    /// If true, truncate inputs that exceed the context window. If false, returns an error.
    pub truncate: Option<bool>,

    /// Number of dimensions to generate embeddings for
    pub dimensions: Option<i64>,

    /// Model keep-alive duration
    pub keep_alive: Option<String>,

    /// Runtime options that control text generation
    pub options: Option<CallOptions>,
}

/// Vector embeddings for the input text
#[derive(Debug, serde::Deserialize)]
pub struct EmbedResponse {
    /// Model that produced the embeddings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Array of vector embeddings
    pub embeddings: Vec<Vec<f64>>,

    /// Total time spent generating in nanoseconds
    pub total_duration: i64,

    /// Load time in nanoseconds
    pub load_duration: i64,

    /// Number of input tokens processed to generate embeddings
    pub prompt_eval_count: i64,
}
