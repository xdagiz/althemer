use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::{AlthemerError, Result};
use dirs::home_dir;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AlthemerConfig {
    #[serde(default = "default_themes_dir")]
    pub themes_dir: Option<PathBuf>,

    #[serde(default = "default_show_preview")]
    pub show_preview: bool,

    #[serde(default = "default_quit_on_select")]
    pub quit_on_select: bool,

    #[serde(default = "default_picker_reversed")]
    pub picker_reversed: bool,

    #[serde(default = "default_picker_sort_results")]
    pub picker_sort_results: bool,
}

impl Default for AlthemerConfig {
    fn default() -> Self {
        Self {
            themes_dir: default_themes_dir(),
            show_preview: default_show_preview(),
            quit_on_select: default_quit_on_select(),
            picker_reversed: default_picker_reversed(),
            picker_sort_results: default_picker_sort_results(),
        }
    }
}

fn default_show_preview() -> bool {
    true
}

fn default_quit_on_select() -> bool {
    false
}

fn default_picker_reversed() -> bool {
    true
}

fn default_picker_sort_results() -> bool {
    false
}

fn default_themes_dir() -> Option<PathBuf> {
    alacritty_config_dir().map(|p| p.join("themes"))
}

pub fn get_althemer_config_dir() -> Result<PathBuf> {
    home_dir()
        .map(|p| p.join(".config").join("althemer"))
        .ok_or_else(|| AlthemerError::ConfigNotFound(PathBuf::from("~/.config/althemer")))
}

pub fn get_config_path() -> Result<PathBuf> {
    Ok(get_althemer_config_dir()?.join("config.json"))
}

pub fn get_themes_dir(custom_path: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = custom_path {
        let mut themes_dir = path.to_path_buf();

        if themes_dir.starts_with("~") {
            themes_dir = PathBuf::from(
                shellexpand::tilde(themes_dir.to_str().expect("cannot convert path to str"))
                    .as_ref(),
            );
        }

        if !themes_dir.exists() {
            return Err(AlthemerError::ThemesDirNotFound(themes_dir));
        }

        return Ok(themes_dir);
    }

    let alacritty_dir = alacritty_config_dir()
        .ok_or_else(|| AlthemerError::ConfigNotFound(PathBuf::from("~/.config/alacritty")))?;
    let themes_dir = alacritty_dir.join("themes");
    if !themes_dir.exists() {
        return Err(AlthemerError::ThemesDirNotFound(themes_dir));
    }

    Ok(themes_dir)
}

fn alacritty_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("alacritty"))
}

fn normalize_config(mut config: AlthemerConfig) -> AlthemerConfig {
    if config
        .themes_dir
        .as_ref()
        .is_some_and(|p| p.as_os_str().is_empty())
    {
        config.themes_dir = None;
    }

    config
}

impl AlthemerConfig {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let config_path = match path {
            Some(p) => p.to_path_buf(),
            None => get_config_path()?,
        };

        if !config_path.exists() {
            return Ok(AlthemerConfig::default());
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: AlthemerConfig = serde_json::from_str(&content)?;
        Ok(normalize_config(config))
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = get_althemer_config_dir()?;
        std::fs::create_dir_all(&config_dir)?;

        let config_path = get_config_path()?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn config_loads_from_file() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = dir.path().join("config.json");

        let json = r#"{
            "themes_dir": "/custom/themes",
            "show_preview": false,
            "quit_on_select": true
        }"#;
        fs::write(&config_path, json).expect("Failed to write config");

        let config = AlthemerConfig::load(Some(&config_path)).expect("Should load config");
        assert_eq!(config.themes_dir, Some(PathBuf::from("/custom/themes")));
    }

    #[test]
    fn config_returns_default_when_missing() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = dir.path().join("config.json");
        let config =
            AlthemerConfig::load(Some(&config_path)).expect("Should handle missing config");
        assert!(config.themes_dir.is_some());
    }

    #[test]
    fn empty_themes_dir_becomes_none() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = dir.path().join("config.json");

        let json = r#"{"themes_dir": ""}"#;
        fs::write(&config_path, json).expect("Failed to write config");

        let config = AlthemerConfig::load(Some(&config_path)).expect("Should load config");
        assert!(config.themes_dir.is_none());
    }

    #[test]
    fn config_saves_and_loads() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = dir.path().join("config.json");

        let config = AlthemerConfig {
            themes_dir: Some(PathBuf::from("/test/themes")),
            ..Default::default()
        };

        let content = serde_json::to_string_pretty(&config).expect("Should serialize");
        fs::write(&config_path, &content).expect("Should write");

        let loaded = AlthemerConfig::load(Some(&config_path)).expect("Should load");
        assert_eq!(loaded, config);
    }

    #[test]
    fn default_config_has_show_preview_true() {
        let config = AlthemerConfig::default();
        assert!(config.show_preview);
    }
}
