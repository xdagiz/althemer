use serde::{Deserialize, Serialize};

use crate::error::{AlthemerError, Result};
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
    dirs::config_dir()
        .map(|p| p.join("alacritty"))
        .ok_or_else(|| AlthemerError::ConfigNotFound(PathBuf::from("~/.config/alacritty")))
}

pub fn get_alacritty_config_path() -> Result<PathBuf> {
    Ok(get_alacritty_config_dir()?.join("alacritty.toml"))
}

pub fn read_config(path: &Path) -> Result<AlacrittyConfig> {
    let content = std::fs::read_to_string(path)?;
    let config = toml::from_str::<AlacrittyConfig>(&content)?;
    Ok(config)
}

pub fn write_config(path: &Path, config: &AlacrittyConfig) -> Result<()> {
    let content = toml::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn get_themes_dir(custom_path: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = custom_path {
        let mut themes_dir = path.to_path_buf();

        if themes_dir.starts_with("~") {
            themes_dir =
                PathBuf::from(shellexpand::tilde(&themes_dir.display().to_string()).as_ref());
        }

        if !themes_dir.exists() {
            return Err(AlthemerError::ThemesDirNotFound(themes_dir));
        }

        return Ok(themes_dir);
    }

    let alacritty_dir = get_alacritty_config_dir()?;
    let themes_dir = alacritty_dir.join("themes");
    if !themes_dir.exists() {
        return Err(AlthemerError::ThemesDirNotFound(themes_dir));
    }

    Ok(themes_dir)
}
