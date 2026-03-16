use std::{io::IsTerminal, path::Path};

use crate::{
    config::alacritty::{
        AlacrittyConfig, GeneralConfig, get_alacritty_config_path, read_config, write_config,
    },
    error::{AppError, Result},
    picker::pick_theme,
    themes::{Theme, get_current_theme, get_theme_by_name, list_themes},
};

/// Switches the active Alacritty theme by updating the config file.
pub fn switch_theme(name: &str, custom_theme_path: Option<&Path>) -> Result<Theme> {
    let theme = get_theme_by_name(name, custom_theme_path)?;
    let config_path = get_alacritty_config_path()?;

    if !config_path.exists() {
        return Err(AppError::ConfigNotFound(config_path));
    }

    let config = read_config(&config_path)?;
    let import_path = theme.path.to_string_lossy().to_string();

    let new_config = AlacrittyConfig {
        general: GeneralConfig {
            import: vec![import_path],
        },
        other: config.other,
    };

    write_config(&config_path, &new_config)?;
    Ok(theme)
}

/// Selects a theme from the list and switches to it
pub fn select_theme(custom_theme_path: Option<&Path>) -> Result<Theme> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(AppError::NoTerminal);
    }

    let themes = list_themes(custom_theme_path)?;
    let current = get_current_theme(custom_theme_path).ok().flatten();

    match pick_theme(&themes, current.as_ref()) {
        Some(theme) => Ok(theme),
        None => Err(AppError::InteractiveError("No theme selected".to_string())),
    }
}
