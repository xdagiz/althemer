use ratatui::style::Color;
use serde::Deserialize;

use crate::config::alacritty::{get_alacritty_config_path, read_config};
use crate::config::configuration::get_themes_dir;
use crate::error::{AlthemerError, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub name_lower: String,
    pub path: PathBuf,
    pub category: ThemeCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeCategory {
    #[default]
    Dark,
    Light,
}

impl ThemeCategory {
    pub fn label(&self) -> &'static str {
        match self {
            ThemeCategory::Dark => "Dark",
            ThemeCategory::Light => "Light",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ThemeCategory::Dark => "⏾",
            ThemeCategory::Light => "☀",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThemeGroups {
    pub dark: Vec<Theme>,
    pub light: Vec<Theme>,
}

impl ThemeGroups {
    pub fn from_themes(mut themes: Vec<Theme>) -> Self {
        for theme in &mut themes {
            theme.categorize().ok();
        }

        let mut dark = Vec::new();
        let mut light = Vec::new();
        for theme in themes {
            match theme.category {
                ThemeCategory::Dark => dark.push(theme),
                ThemeCategory::Light => light.push(theme),
            }
        }

        Self { dark, light }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ThemeColors {
    #[serde(default)]
    pub colors: ColorScheme,
    #[serde(skip)]
    cached: Option<CachedColors>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ColorScheme {
    #[serde(default, rename = "primary")]
    pub primary_hex: PrimaryColorsHex,
    #[serde(default, rename = "cursor")]
    pub cursor_hex: CursorColorsHex,
    #[serde(default)]
    pub normal: NormalColors,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PrimaryColorsHex {
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub foreground: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct CachedColors {
    background: Option<Color>,
    foreground: Option<Color>,
    green: Option<Color>,
    yellow: Option<Color>,
    blue: Option<Color>,
    magenta: Option<Color>,
    cyan: Option<Color>,
    cursor_text: Option<Color>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CursorColorsHex {
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
        let mut colors: ThemeColors = toml::from_str(&content)
            .map_err(|e| AlthemerError::PreviewError(format!("Failed to parse theme: {}", e)))?;

        let cs = &colors.colors;
        colors.cached = Some(CachedColors {
            background: cs
                .primary_hex
                .background
                .as_ref()
                .map(|h| Self::hex_to_ratatui(h)),
            foreground: cs
                .primary_hex
                .foreground
                .as_ref()
                .map(|h| Self::hex_to_ratatui(h)),
            green: Self::hex_to_ratatui_opt(&cs.normal.green),
            yellow: Self::hex_to_ratatui_opt(&cs.normal.yellow),
            blue: Self::hex_to_ratatui_opt(&cs.normal.blue),
            magenta: Self::hex_to_ratatui_opt(&cs.normal.magenta),
            cyan: Self::hex_to_ratatui_opt(&cs.normal.cyan),
            cursor_text: Self::hex_to_ratatui_opt(&cs.cursor_hex.text),
        });

        Ok(colors)
    }

    fn hex_to_ratatui_opt(hex: &str) -> Option<Color> {
        let hex = hex.trim();
        if hex.is_empty() {
            return None;
        }

        let hex = hex.trim_start_matches('#');
        if hex.len() < 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Color::Rgb(r, g, b))
    }

    fn hex_to_ratatui(hex: &str) -> Color {
        Self::hex_to_ratatui_opt(hex).unwrap_or(Color::Reset)
    }

    pub fn background(&self) -> Color {
        self.cached
            .as_ref()
            .and_then(|c| c.background)
            .unwrap_or(Color::Reset)
    }

    pub fn foreground(&self) -> Color {
        self.cached
            .as_ref()
            .and_then(|c| c.foreground)
            .unwrap_or(Color::Reset)
    }

    pub fn green(&self) -> Color {
        self.cached
            .as_ref()
            .and_then(|c| c.green)
            .unwrap_or(Color::Reset)
    }

    pub fn yellow(&self) -> Color {
        self.cached
            .as_ref()
            .and_then(|c| c.yellow)
            .unwrap_or(Color::Reset)
    }

    pub fn blue(&self) -> Color {
        self.cached
            .as_ref()
            .and_then(|c| c.blue)
            .unwrap_or(Color::Reset)
    }

    pub fn magenta(&self) -> Color {
        self.cached
            .as_ref()
            .and_then(|c| c.magenta)
            .unwrap_or(Color::Reset)
    }

    pub fn cyan(&self) -> Color {
        self.cached
            .as_ref()
            .and_then(|c| c.cyan)
            .unwrap_or(Color::Reset)
    }

    pub fn cursor_text(&self) -> Color {
        self.cached
            .as_ref()
            .and_then(|c| c.cursor_text)
            .unwrap_or(Color::Reset)
    }
}

impl Theme {
    /// Creates a Theme from a file path.
    pub fn from_path(path: &Path) -> Option<Self> {
        let name = path.file_stem()?.to_str()?.to_string();
        let name_lower = name.to_lowercase();
        Some(Theme {
            name,
            name_lower,
            path: path.to_path_buf(),
            category: ThemeCategory::default(),
        })
    }

    pub fn categorize(&mut self) -> Result<()> {
        if let Some(category) = self.categorize_from_filename() {
            self.category = category;
            return Ok(());
        }
        Ok(())
    }

    fn categorize_from_filename(&self) -> Option<ThemeCategory> {
        let name_lower = &self.name_lower;

        if name_lower.ends_with("_dark") || name_lower.ends_with("-dark") {
            return None;
        }

        const LIGHT_KEYWORDS: &[&str] = &[
            "_light",
            "-light",
            "acme",
            "alabaster",
            "latte",
            "dayfox",
            "morningfox",
            "noctis_lux",
            "papertheme",
            "dawn",
        ];

        if LIGHT_KEYWORDS.iter().any(|&kw| name_lower.contains(kw)) {
            return Some(ThemeCategory::Light);
        }

        None
    }
}

/// Lists all available themes from the themes directory.
pub fn list_themes(custom_path: Option<&Path>) -> Result<Vec<Theme>> {
    let themes_dir = get_themes_dir(custom_path)?;
    let themes_dir_canonical = themes_dir.canonicalize().ok();

    let mut themes: Vec<Theme> = std::fs::read_dir(&themes_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|ext| ext == "toml")
                .unwrap_or(false)
        })
        .filter_map(|entry| {
            let path = entry.path();
            match (&themes_dir_canonical, path.canonicalize().ok()) {
                (Some(dir), Some(canonical)) if !canonical.starts_with(dir) => None,
                _ => Theme::from_path(&path),
            }
        })
        .collect();

    themes.sort_by(|a, b| a.name.cmp(&b.name));
    let groups = ThemeGroups::from_themes(themes);
    let themes: Vec<Theme> = groups.dark.into_iter().chain(groups.light).collect();

    Ok(themes)
}

/// Gets the currently active theme from the Alacritty config.
pub fn get_current_theme(custom_path: Option<&Path>) -> Result<Option<Theme>> {
    let config_path = get_alacritty_config_path()?;

    if !config_path.exists() {
        return Err(AlthemerError::ConfigNotFound(config_path));
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
        name_lower: theme_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_lowercase(),
        path: theme_path,
        category: ThemeCategory::Dark,
    }))
}

/// Looks up a theme by name (exact match or case-insensitive).
pub fn get_theme_by_name(name: &str, custom_path: Option<&Path>) -> Result<Theme> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AlthemerError::ThemeNotFound(name.to_string()));
    }

    let themes = list_themes(custom_path)?;

    // Try exact match first, then case-insensitive
    themes
        .into_iter()
        .find(|t| t.name == name || t.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| AlthemerError::ThemeNotFound(name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_dir(files: &[&str]) -> TempDir {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        for file in files {
            let path = dir.path().join(file).with_extension("toml");
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
