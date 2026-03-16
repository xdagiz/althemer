use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};
use dirs::home_dir;

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
pub struct AlthemerConfig {
    #[serde(default)]
    pub themes_dir: Option<PathBuf>,
}

pub fn get_althemer_config_dir() -> Result<PathBuf> {
    home_dir()
        .map(|p| p.join(".config").join("althemer"))
        .ok_or_else(|| AppError::ConfigNotFound(PathBuf::from("~/.config/althemer")))
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
            return Err(AppError::ThemesDirNotFound(themes_dir));
        }

        return Ok(themes_dir);
    }

    let alacritty_dir = alacritty_config_dir()
        .ok_or_else(|| AppError::ConfigNotFound(PathBuf::from("~/.config/alacritty")))?;
    let themes_dir = alacritty_dir.join("themes").join("themes");
    if !themes_dir.exists() {
        return Err(AppError::ThemesDirNotFound(themes_dir));
    }

    Ok(themes_dir)
}

fn alacritty_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("alacritty"))
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

        Ok(config)
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
            "show_preview": false
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
        assert_eq!(config.themes_dir, None);
    }

    #[test]
    fn config_saves_and_loads() {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = dir.path().join("config.json");

        let config = AlthemerConfig {
            themes_dir: Some(PathBuf::from("/test/themes")),
        };

        let content = serde_json::to_string_pretty(&config).expect("Should serialize");
        fs::write(&config_path, &content).expect("Should write");

        let loaded = AlthemerConfig::load(Some(&config_path)).expect("Should load");
        assert_eq!(loaded, config);
    }
}
