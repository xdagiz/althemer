use std::{
    io::{self, IsTerminal},
    path::Path,
};

use crate::{
    config::{get_alacritty_config_path, get_themes_dir, read_config, write_config},
    error::{AppError, Result},
    picker::pick_theme,
    themes::{Theme, get_current_theme, get_theme_by_name, list_themes},
};

/// Switches the active Alacritty theme by updating the config file.
pub fn switch_theme(name: &str, custom_theme_path: Option<&Path>) -> Result<Theme> {
    let theme = get_theme_by_name(name, custom_theme_path)?;
    let config_path = get_alacritty_config_path()?;

    if !config_path.exists() {
        return Err(crate::error::AppError::ConfigNotFound(config_path));
    }

    let mut config = read_config(&config_path)?;
    let theme_path_str = theme.path.to_string_lossy();
    config.general.import = vec![theme_path_str.to_string()];

    write_config(&config_path, &config)?;

    Ok(theme)
}

/// Selects a theme from the list and switches to it
pub fn select_theme(custom_path: Option<&Path>) -> Result<Option<Theme>> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(AppError::NoTerminal);
    }

    let themes = list_themes(custom_path)?;

    if themes.is_empty() {
        let themes_dir = get_themes_dir(custom_path)?;
        return Err(AppError::ThemesDirNotFound(themes_dir));
    }

    if themes.len() == 1 {
        let theme = &themes[0];
        let switched = switch_theme(&theme.name, custom_path)?;
        println!("✓ Only one theme available. Switched to: {}", theme.name);
        return Ok(Some(switched));
    }

    let current_theme = get_current_theme(custom_path).ok().flatten();

    if current_theme.is_none() {
        eprintln!("Warning: Could not determine current theme");
    }

    let selected_theme = pick_theme(&themes, current_theme.as_ref())
        .ok_or_else(|| AppError::InteractiveError("No theme selected".to_string()))?;

    switch_theme(&selected_theme.name, custom_path)?;
    println!("✓ Switched to theme: {}", selected_theme.name);

    Ok(Some(selected_theme))
}
