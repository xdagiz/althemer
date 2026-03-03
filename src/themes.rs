use crate::config::get_themes_dir;
use crate::error::Result;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
}

impl Theme {
    pub fn from_path(path: &Path) -> Option<Self> {
        let file_stem = path.file_stem()?.to_str()?;
        Some(Theme {
            name: file_stem.to_string(),
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
