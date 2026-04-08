//! Configuration module for PC Controller
//!
//! Loads configuration from environment variables and optional config file.

use serde::Deserialize;
use std::path::PathBuf;
use std::sync::RwLock;

/// Vision model configuration
#[derive(Debug, Clone, Deserialize)]
pub struct VisionConfig {
    /// OpenAI API key
    pub api_key: String,
    /// Base URL for OpenAI API (optional)
    #[serde(default = "default_base_url")]
    pub base_url: String,
    /// Model name (default: gpt-4-vision-preview)
    #[serde(default = "default_model")]
    pub model: String,
}

fn default_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_model() -> String {
    "gpt-4-vision-preview".to_string()
}

impl Default for VisionConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            base_url: std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| default_base_url()),
            model: std::env::var("OPENAI_VISION_MODEL").unwrap_or_else(|_| default_model()),
        }
    }
}

impl VisionConfig {
    /// Load from environment variables
    pub fn from_env() -> Self {
        Self::default()
    }

    /// Load from a TOML config file, with environment fallback
    pub fn from_file(path: &PathBuf) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::FileRead(e.to_string()))?;

        let mut config: VisionConfig = toml_edit::de::from_str(&contents)
            .map_err(|e| ConfigError::Parse(e.to_string()))?;

        // Environment variables override file settings
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            config.api_key = api_key;
        }
        if let Ok(base_url) = std::env::var("OPENAI_BASE_URL") {
            config.base_url = base_url;
        }
        if let Ok(model) = std::env::var("OPENAI_VISION_MODEL") {
            config.model = model;
        }

        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.api_key.is_empty() {
            return Err(ConfigError::MissingApiKey);
        }
        Ok(())
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileRead(String),
    #[error("Failed to parse config: {0}")]
    Parse(String),
    #[error("Missing API key: set OPENAI_API_KEY environment variable or api_key in config")]
    MissingApiKey,
}

/// Global config state
static CONFIG: RwLock<Option<VisionConfig>> = RwLock::new(None);

/// Initialize global config
pub fn init(config: VisionConfig) -> Result<(), ConfigError> {
    config.validate()?;
    let mut global = CONFIG.write().unwrap();
    *global = Some(config);
    Ok(())
}

/// Get global config
pub fn get_config() -> Option<VisionConfig> {
    CONFIG.read().unwrap().clone()
}
