use ratatui::style::Color;
use serde::Deserialize;

use crate::config::{get_alacritty_config_path, get_themes_dir, read_config};
use crate::error::{AppError, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ThemeColors {
    #[serde(default)]
    pub colors: ColorScheme,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ColorScheme {
    #[serde(default)]
    pub primary: PrimaryColors,
    #[serde(default)]
    pub cursor: CursorColors,
    #[serde(default)]
    pub normal: NormalColors,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PrimaryColors {
    #[serde(default)]
    pub background: String,
    #[serde(default)]
    pub foreground: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CursorColors {
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct NormalColors {
    #[serde(default)]
    pub green: String,
    #[serde(default)]
    pub yellow: String,
    #[serde(default)]
    pub blue: String,
    #[serde(default)]
    pub magenta: String,
    #[serde(default)]
    pub cyan: String,
}

impl ThemeColors {
    pub fn from_path(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let colors: ThemeColors = toml::from_str(&content)
            .map_err(|e| AppError::PreviewError(format!("Failed to parse theme: {}", e)))?;

        Ok(colors)
    }

    fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
        let hex = hex.trim_start_matches('#');
        if hex.len() < 6 {
            return (0, 0, 0);
        }

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

        (r, g, b)
    }

    pub fn hex_to_ratatui(hex: &str) -> Color {
        let (r, g, b) = Self::hex_to_rgb(hex);
        Color::Rgb(r, g, b)
    }

    pub fn background(&self) -> Color {
        Self::hex_to_ratatui(&self.colors.primary.background)
    }

    pub fn foreground(&self) -> Color {
        Self::hex_to_ratatui(&self.colors.primary.foreground)
    }

    pub fn green(&self) -> Color {
        Self::hex_to_ratatui(&self.colors.normal.green)
    }

    pub fn yellow(&self) -> Color {
        Self::hex_to_ratatui(&self.colors.normal.yellow)
    }

    pub fn blue(&self) -> Color {
        Self::hex_to_ratatui(&self.colors.normal.blue)
    }

    pub fn magenta(&self) -> Color {
        Self::hex_to_ratatui(&self.colors.normal.magenta)
    }

    pub fn cyan(&self) -> Color {
        Self::hex_to_ratatui(&self.colors.normal.cyan)
    }

    pub fn cursor_text(&self) -> Color {
        Self::hex_to_ratatui(&self.colors.cursor.text)
    }
}

impl Theme {
    /// Creates a Theme from a file path.
    pub fn from_path(path: &Path) -> Option<Self> {
        let name = path.file_stem()?.to_str()?.to_string();
        Some(Theme {
            name,
            path: path.to_path_buf(),
        })
    }
}

/// Lists all available themes from the themes directory.
pub fn list_themes(custom_path: Option<&Path>) -> Result<Vec<Theme>> {
    let themes_dir = get_themes_dir(custom_path)?;

    let mut themes: Vec<Theme> = std::fs::read_dir(themes_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|ext| ext == "toml")
                .unwrap_or(false)
        })
        .filter_map(|entry| Theme::from_path(&entry.path()))
        .collect();

    themes.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(themes)
}

/// Gets the currently active theme from the Alacritty config.
pub fn get_current_theme(custom_path: Option<&Path>) -> Result<Option<Theme>> {
    let config_path = get_alacritty_config_path()?;

    if !config_path.exists() {
        return Err(AppError::ConfigNotFound(config_path));
    }

    let config = read_config(&config_path)?;

    if config.general.import.is_empty() {
        return Ok(None);
    }

    let theme_path = PathBuf::from(&config.general.import[0]);
    let themes = list_themes(custom_path)?;

    if let Some(theme) = themes.into_iter().find(|t| t.path == theme_path) {
        return Ok(Some(theme));
    }

    Ok(Some(Theme {
        name: theme_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string(),
        path: theme_path,
    }))
}

/// Looks up a theme by name (exact match or case-insensitive).
pub fn get_theme_by_name(name: &str, custom_path: Option<&Path>) -> Result<Theme> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::ThemeNotFound(name.to_string()));
    }

    let themes = list_themes(custom_path)?;

    // Try exact match first, then case-insensitive
    themes
        .into_iter()
        .find(|t| t.name == name || t.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| AppError::ThemeNotFound(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_dir(files: &[&str]) -> TempDir {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        for file in files {
            let path = dir.path().join(format!("{}.toml", file));
            fs::write(&path, "").expect("Failed to write temp file");
        }
        dir
    }

    #[test]
    fn theme_from_path_extracts_name_from_filename() {
        let dir = create_temp_dir(&["dracula", "nord"]);
        let theme_path = dir.path().join("dracula.toml");

        let theme = Theme::from_path(&theme_path).expect("Should parse theme");

        assert_eq!(theme.name, "dracula");
        assert_eq!(theme.path, theme_path);
    }

    #[test]
    fn theme_from_path_works_on_any_file() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = dir.path().join("theme.toml");
        fs::write(&path, "").expect("Failed to write temp file");

        let result = Theme::from_path(&path);

        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "theme");
    }

    #[test]
    fn theme_name_preserves_original_case() {
        let dir = create_temp_dir(&["SolarizedDark"]);
        let theme_path = dir.path().join("SolarizedDark.toml");

        let theme = Theme::from_path(&theme_path).expect("Should parse theme");

        assert_eq!(theme.name, "SolarizedDark");
    }

    #[test]
    fn list_themes_filters_only_toml_files() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        fs::write(dir.path().join("valid.toml"), "").expect("Failed to write file");
        fs::write(dir.path().join("readme.txt"), "").expect("Failed to write file");
        fs::write(dir.path().join("config.yml"), "").expect("Failed to write file");

        let themes = list_themes(Some(dir.path())).expect("Should list themes");

        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].name, "valid");
    }

    #[test]
    fn get_theme_by_name_exact_match() {
        let dir = create_temp_dir(&["dracula", "nord", "gruvbox"]);

        let result = get_theme_by_name("dracula", Some(dir.path()));

        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "dracula");
    }

    #[test]
    fn get_theme_by_name_case_insensitive() {
        let dir = create_temp_dir(&["Dracula", "Nord"]);

        let result = get_theme_by_name("dracula", Some(dir.path()));

        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Dracula");
    }

    #[test]
    fn get_theme_by_name_not_found() {
        let dir = create_temp_dir(&["dracula"]);

        let result = get_theme_by_name("nonexistent", Some(dir.path()));

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("nonexistent"));
    }
}
