use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlacrittyConfig {
    #[serde(default)]
    pub general: GeneralConfig,

    #[serde(flatten)]
    pub other: toml::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneralConfig {
    #[serde(default)]
    pub import: Vec<String>,
}

/// Returns the Alacritty config directory (~/.config/alacritty).
pub fn get_alacritty_config_dir() -> Result<PathBuf> {
    dirs::config_dir()
        .map(|p| p.join("alacritty"))
        .ok_or_else(|| AppError::ConfigNotFound(PathBuf::from("~/.config/alacritty")))
}

/// Returns the path to alacritty.toml config file.
pub fn get_alacritty_config_path() -> Result<PathBuf> {
    Ok(get_alacritty_config_dir()?.join("alacritty.toml"))
}

/// Returns the themes directory, either from custom path or default location.
pub fn get_themes_dir(custom_path: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = custom_path {
        let themes_dir = path.to_path_buf();
        if !themes_dir.exists() {
            return Err(AppError::ThemesDirNotFound(themes_dir));
        }

        return Ok(themes_dir);
    }

    let themes_dir = get_alacritty_config_dir()?.join("themes").join("themes");

    if !themes_dir.exists() {
        return Err(AppError::ThemesDirNotFound(themes_dir));
    }

    Ok(themes_dir)
}

/// Reads and parses an Alacritty config file.
pub fn read_config(path: &Path) -> Result<AlacrittyConfig> {
    let content = std::fs::read_to_string(path)?;
    let config = toml::from_str::<AlacrittyConfig>(&content)?;
    Ok(config)
}

/// Writes an Alacritty config file.
pub fn write_config(path: &Path, config: &AlacrittyConfig) -> Result<()> {
    let content = toml::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}
