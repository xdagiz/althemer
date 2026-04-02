use ratatui::style::Color;
use relative_luminance::{Luminance, Rgb};
use serde::Deserialize;

use crate::alacritty::{get_alacritty_config_path, get_themes_dir, read_config};
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
    pub fn label(self) -> &'static str {
        match self {
            ThemeCategory::Dark => "Dark",
            ThemeCategory::Light => "Light",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            ThemeCategory::Dark => "⏾",
            ThemeCategory::Light => "☀",
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ThemeColors {
    #[serde(default)]
    pub colors: ColorScheme,
    #[serde(skip)]
    cached: Option<CachedColors>,
}

macro_rules! impl_color_accessors {
    ($($name:ident),*) => {
        $(
            pub fn $name(&self) -> Color {
                self.cached
                    .as_ref()
                    .and_then(|c| c.$name)
                    .unwrap_or(Color::Reset)
            }
        )*
    };
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
    red: Option<Color>,
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
    pub red: String,
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
            .map_err(|e| AlthemerError::PreviewError(format!("Failed to parse theme: {e}")))?;

        let cs = &colors.colors;
        colors.cached = Some(CachedColors {
            background: cs
                .primary_hex
                .background
                .as_ref()
                .map(|s| hex_to_ratatui(s)),
            foreground: cs
                .primary_hex
                .foreground
                .as_ref()
                .map(|s| hex_to_ratatui(s)),
            red: hex_to_ratatui_opt(&cs.normal.red),
            green: hex_to_ratatui_opt(&cs.normal.green),
            yellow: hex_to_ratatui_opt(&cs.normal.yellow),
            blue: hex_to_ratatui_opt(&cs.normal.blue),
            magenta: hex_to_ratatui_opt(&cs.normal.magenta),
            cyan: hex_to_ratatui_opt(&cs.normal.cyan),
            cursor_text: hex_to_ratatui_opt(&cs.cursor_hex.text),
        });

        Ok(colors)
    }

    impl_color_accessors!(
        background,
        foreground,
        red,
        green,
        yellow,
        blue,
        magenta,
        cyan,
        cursor_text
    );
}

fn hex_to_ratatui_opt(hex: &str) -> Option<Color> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() < 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Color::Rgb(r, g, b))
}

fn hex_to_ratatui(hex: &str) -> Color {
    hex_to_ratatui_opt(hex).unwrap_or(Color::Reset)
}

pub fn parse_hex_color(hex: &str) -> Option<Rgb<f64>> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() < 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Rgb {
        r: f64::from(r) / 255.0,
        g: f64::from(g) / 255.0,
        b: f64::from(b) / 255.0,
    })
}

impl Theme {
    pub fn from_path(path: &Path) -> Option<Self> {
        let name = path.file_stem()?.to_str()?.to_string();
        let name_lower = name.to_lowercase();

        let mut theme = Self {
            name_lower,
            name,
            path: path.to_path_buf(),
            category: ThemeCategory::default(),
        };

        theme.category = theme.categorize();
        Some(theme)
    }

    fn categorize(&self) -> ThemeCategory {
        self.categorize_from_filename()
            .unwrap_or_else(|| self.categorize_from_luminance().unwrap_or_default())
    }

    fn categorize_from_filename(&self) -> Option<ThemeCategory> {
        let name_lower = &self.name_lower;

        if name_lower.ends_with("_dark") || name_lower.ends_with("-dark") {
            return Some(ThemeCategory::Dark);
        }

        if name_lower.ends_with("_light") || name_lower.ends_with("-light") {
            return Some(ThemeCategory::Light);
        }

        None
    }

    fn categorize_from_luminance(&self) -> Option<ThemeCategory> {
        let content = std::fs::read_to_string(&self.path).ok()?;
        let parsed: toml::Value = toml::from_str(&content).ok()?;

        let hex = parsed
            .get("colors")
            .and_then(|c| c.get("primary"))
            .and_then(|p| p.get("background"))
            .and_then(|v| v.as_str())?;

        let rgb = parse_hex_color(hex)?;
        let luminance = rgb.relative_luminance();

        if luminance > 0.5 {
            Some(ThemeCategory::Light)
        } else {
            None
        }
    }
}

pub fn list_themes(custom_path: Option<&Path>) -> Result<Vec<Theme>> {
    let themes_dir = get_themes_dir(custom_path)?;

    let mut themes = std::fs::read_dir(themes_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "toml"))
        .filter_map(|entry| Theme::from_path(&entry.path()))
        .collect::<Vec<_>>();

    themes.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(themes)
}

pub fn get_current_theme_path() -> Result<Option<PathBuf>> {
    let config_path = get_alacritty_config_path()?;

    if !config_path.exists() {
        return Err(AlthemerError::ConfigNotFound(config_path));
    }

    let config = read_config(&config_path)?;

    Ok(config.general.import.first().map(PathBuf::from))
}

pub fn get_current_theme(custom_path: Option<&Path>) -> Result<Option<Theme>> {
    let Some(theme_path) = get_current_theme_path()? else {
        return Ok(None);
    };
    let themes = list_themes(custom_path)?;

    Ok(themes.into_iter().find(|t| t.path == theme_path))
}

pub fn get_theme_by_name(name: &str, custom_path: Option<&Path>) -> Result<Theme> {
    let themes = list_themes(custom_path)?;
    let name_lower = name.to_lowercase();

    themes
        .into_iter()
        .find(|t| t.name_lower == name_lower)
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
        fs::write(&path, "").expect("Failed to write theme file");

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

    #[test]
    fn parse_hex_color_valid() {
        let rgb = parse_hex_color("#282a36").unwrap();
        assert!((rgb.r - 0.157).abs() < 0.01);
        assert!((rgb.g - 0.165).abs() < 0.01);
        assert!((rgb.b - 0.212).abs() < 0.01);
    }

    #[test]
    fn parse_hex_color_invalid_chars() {
        assert!(parse_hex_color("#xyz123").is_none());
    }

    #[test]
    fn parse_hex_color_no_hash() {
        assert!(parse_hex_color("282a36").is_some());
    }

    #[test]
    fn parse_hex_color_black() {
        let rgb = parse_hex_color("#000000").unwrap();
        assert!(rgb.r < 0.001);
        assert!(rgb.g < 0.001);
        assert!(rgb.b < 0.001);
    }

    #[test]
    fn parse_hex_color_white() {
        let rgb = parse_hex_color("#ffffff").unwrap();
        assert!(rgb.r > 0.999);
        assert!(rgb.g > 0.999);
        assert!(rgb.b > 0.999);
    }

    fn create_theme_file(dir: &TempDir, name: &str, toml_content: &str) -> Theme {
        let path = dir.path().join(format!("{name}.toml"));
        fs::write(&path, toml_content).expect("Failed to write theme file");
        Theme::from_path(&path).unwrap()
    }

    #[test]
    fn light_background_categorizes_as_light() {
        let dir = tempfile::tempdir().unwrap();
        let theme = create_theme_file(
            &dir,
            "test_light",
            r##"
[colors.primary]
background = "#fbf1c7"
foreground = "#3c3836"
"##,
        );
        assert_eq!(theme.category, ThemeCategory::Light);
    }

    #[test]
    fn dark_background_categorizes_as_dark() {
        let dir = tempfile::tempdir().unwrap();
        let theme = create_theme_file(
            &dir,
            "test_dark",
            r##"
[colors.primary]
background = "#282a36"
foreground = "#f8f8f2"
"##,
        );
        assert_eq!(theme.category, ThemeCategory::Dark);
    }

    #[test]
    fn filename_suffix_overrides_luminance() {
        let dir = tempfile::tempdir().unwrap();
        let theme = create_theme_file(
            &dir,
            "custom_dark",
            r##"
[colors.primary]
background = "#fbf1c7"
foreground = "#3c3836"
"##,
        );
        assert_eq!(theme.category, ThemeCategory::Dark);
    }

    #[test]
    fn no_colors_section_stays_default_dark() {
        let dir = tempfile::tempdir().unwrap();
        let theme = create_theme_file(
            &dir,
            "broken",
            r##"
some_other_field = "value"
"##,
        );
        assert_eq!(theme.category, ThemeCategory::Dark);
    }

    #[test]
    fn light_suffix_categorizes_as_light() {
        let dir = tempfile::tempdir().unwrap();
        let theme = create_theme_file(&dir, "gruvbox_light", r#""#);
        assert_eq!(theme.category, ThemeCategory::Light);
    }

    #[test]
    fn dark_suffix_stays_dark() {
        let dir = tempfile::tempdir().unwrap();
        let theme = create_theme_file(&dir, "gruvbox_dark", r#""#);
        assert_eq!(theme.category, ThemeCategory::Dark);
    }
}
