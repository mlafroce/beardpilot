/// List available models
#[derive(Debug, serde::Deserialize)]
pub struct TagList {
    pub models: Vec<TagInfo>,
}

#[derive(Debug, serde::Deserialize)]
pub struct TagInfo {
    /// Model name
    pub name: String,
    /// Model name
    pub model: String,
    /// Name of the upstream model, if the model is remote
    pub remote_model: Option<String>,
    /// URL of the upstream Ollama host, if the model is remote
    pub remote_host: Option<String>,
    /// Last modified timestamp in ISO 8601 format
    pub modified_at: String,
    /// Total size of the model on disk in bytes
    pub size: u32,
    /// SHA256 digest identifier of the model contents
    pub digest: String,
    /// Additional information about the model's format and family
    pub details: TagDetails,
}

#[derive(Debug, serde::Deserialize)]
pub struct TagDetails {
    /// Model file format (for example gguf)
    pub format: String,
    /// Primary model family (for example llama)
    pub family: String,
    /// All families the model belongs to, when applicable
    pub families: Vec<String>,
    /// Approximate parameter count label (for example 7B, 13B)
    pub parameter_size: String,
    /// Quantization level used (for example Q4_0)
    pub quantization_level: String,
}
