use serde_json::Value;

/// Models currently loaded into memory
#[derive(Debug, serde::Deserialize)]
pub struct ModelList {
    /// Currently running models
    pub models: Vec<Model>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Model {
    /// Name of the running model
    pub name: String,
    /// Name of the running model
    pub model: String,
    /// Size of the model in bytes
    pub size: i64,
    /// SHA256 digest of the model
    pub digest: String,
    /// Model details such as format and family
    pub details: Value,
    /// Time when the model will be unloaded
    pub expires_at: String,
    /// VRAM usage in bytes
    pub size_vram: i64,
    /// Context length for the running model
    pub context_length: i32,
}
