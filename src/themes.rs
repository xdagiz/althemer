use crate::config::{get_alacritty_config_dir, get_themes_dir, read_config};
use crate::error::{AppError, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub path: PathBuf,
}

impl Theme {
    pub fn from_path(path: &Path) -> Option<Self> {
        let file_stem = path.file_stem()?.to_str()?;
        Some(Theme {
            name: file_stem.to_string(),
            path: path.to_path_buf(),
        })
    }
}

pub fn list_themes() -> Result<Vec<Theme>> {
    // ~/.config/alacritty/themes/themes
    let themes_dir = get_themes_dir()?;

    let mut themes: Vec<Theme> = std::fs::read_dir(themes_dir)?
        // verify the directory exists
        .filter_map(|entry| entry.ok())
        // filter .toml files
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|ext| ext == "toml")
                .unwrap_or(false)
        })
        // get the name of each theme
        .filter_map(|entry| Theme::from_path(&entry.path()))
        .collect();

    themes.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(themes)
}

pub fn get_current_theme() -> Result<Option<Theme>> {
    let config_path = get_alacritty_config_dir()?;

    if !config_path.exists() {
        return Err(AppError::ConfigNotFound(config_path));
    }

    let config_path = config_path.join("alacritty.toml");
    let config = read_config(&config_path)?;

    if config.general.import.is_empty() {
        return Ok(None);
    }

    let theme_path = &config.general.import[0];
    let theme_path = PathBuf::from(theme_path);
    let themes = list_themes()?;

    if let Some(theme) = themes.iter().find(|&t| t.path == theme_path) {
        return Ok(Some(theme.clone()));
    }

    Ok(Some(Theme {
        name: theme_path
            .as_path()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string(),
        path: theme_path,
    }))
}
