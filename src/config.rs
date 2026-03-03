use crate::error::{AppError, Result};
use std::path::PathBuf;

pub fn get_alacritty_config_dir() -> Result<PathBuf> {
    // ~/.config
    dirs::config_dir()
        .map(|p| p.join("alacritty"))
        // default to ~/.config/alacritty
        .ok_or_else(|| AppError::ConfigNotFound(PathBuf::from("~/.config/alacritty")))
}

pub fn get_themes_dir() -> Result<PathBuf> {
    // ~/.config/alacritty/themes/themes
    let themes_dir = get_alacritty_config_dir()?.join("themes").join("themes");

    if !themes_dir.exists() {
        return Err(AppError::ThemesDirNotFound(themes_dir));
    }

    Ok(themes_dir)
}
