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
}

pub type Result<T> = std::result::Result<T, AppError>;
