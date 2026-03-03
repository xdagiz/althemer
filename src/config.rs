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

pub fn get_alacritty_config_dir() -> Result<PathBuf> {
    // ~/.config
    dirs::config_dir()
        .map(|p| p.join("alacritty"))
        // default to ~/.config/alacritty
        .ok_or_else(|| AppError::ConfigNotFound(PathBuf::from("~/.config/alacritty")))
}

pub fn get_alacritty_config_path() -> Result<PathBuf> {
    Ok(get_alacritty_config_dir()?.join("alacritty.toml"))
}

pub fn get_themes_dir() -> Result<PathBuf> {
    // ~/.config/alacritty/themes/themes
    let themes_dir = get_alacritty_config_dir()?.join("themes").join("themes");

    if !themes_dir.exists() {
        return Err(AppError::ThemesDirNotFound(themes_dir));
    }

    Ok(themes_dir)
}

pub fn read_config(path: &Path) -> Result<AlacrittyConfig> {
    let content = std::fs::read_to_string(path)?;
    let config = toml::from_str::<AlacrittyConfig>(&content)?;
    Ok(config)
}

pub fn write_config(path: &PathBuf, config: &AlacrittyConfig) -> Result<()> {
    let content = toml::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}
