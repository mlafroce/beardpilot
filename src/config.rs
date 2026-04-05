use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::AppError;

const CONFIG_FOLDER: &str = "beardpilot";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    /// Ollama model identifier to use for chat
    pub model: String,
    /// Ollama server host (e.g. "http://localhost")
    pub host: String,
    /// Ollama server port
    pub port: u16,
    /// Optional system prompt injected at the start of every conversation
    pub system_prompt: Option<String>,
    /// Maximum number of messages kept in history (oldest are dropped first).
    /// `None` means unlimited.
    pub max_history: Option<usize>,
    /// Maximum number of tokens the model can generate in a single response.
    /// `None` means the model's default is used.
    pub max_tokens: Option<usize>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model: "qwen3.5-4b-8k:latest".to_string(),
            host: "http://localhost".to_string(),
            port: 11434,
            system_prompt: None,
            max_history: None,
            max_tokens: None,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, AppError> {
        let mut config = match Self::config_path() {
            Some(path) if path.exists() => {
                let contents = std::fs::read_to_string(&path)
                    .map_err(|e| AppError::Config(format!("Cannot read {:?}: {}", path, e)))?;
                toml::from_str(&contents)
                    .map_err(|e| AppError::Config(format!("Invalid config in {:?}: {}", path, e)))
            }
            Some(path) => {
                // Config dir is known but the file doesn't exist yet — create it with defaults.
                let default = Self::default();
                let toml_str = toml::to_string_pretty(&default).map_err(|e| {
                    AppError::Config(format!("Cannot serialize default config: {}", e))
                })?;
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        AppError::Config(format!("Cannot create config dir {:?}: {}", parent, e))
                    })?;
                }
                std::fs::write(&path, &toml_str).map_err(|e| {
                    AppError::Config(format!("Cannot write default config to {:?}: {}", path, e))
                })?;
                Ok(default)
            }
            None => Ok(Self::default()),
        }?;

        // Override system_prompt with AGENTS.md if present (takes priority over config.toml).
        if let Some(agents_path) = Self::agents_md_path() {
            if agents_path.exists() {
                let prompt = std::fs::read_to_string(&agents_path).map_err(|e| {
                    AppError::Config(format!("Cannot read {:?}: {}", agents_path, e))
                })?;
                config.system_prompt = Some(prompt);
            }
        }

        Ok(config)
    }

    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join(CONFIG_FOLDER).join("config.toml"))
    }

    fn agents_md_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join(CONFIG_FOLDER).join("SYSTEM.md"))
    }
}
