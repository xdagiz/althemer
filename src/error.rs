use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Failed to read directory: {0}")]
    DirectoryRead(#[from] io::Error),

    #[error("Config file not found at: {0}")]
    ConfigNotFound(PathBuf),

    #[error("Themes directory not found at: {0}")]
    ThemesDirNotFound(PathBuf),

    #[error("Theme not found: {0}")]
    ThemeNotFound(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Failed to parse TOML: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Failed to serialize TOML: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Interactive search failed: {0}")]
    InteractiveError(String),

    #[error("Interactive mode requires a terminal")]
    NoTerminal,

    #[error("Failed to render preview: {0}")]
    PreviewError(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
