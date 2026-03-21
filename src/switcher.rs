use std::{io::IsTerminal, path::Path};

use crate::{
    config::AlthemerConfig,
    config::alacritty::{
        AlacrittyConfig, GeneralConfig, get_alacritty_config_path, read_config, write_config,
    },
    error::{AlthemerError, Result},
    picker::pick_theme,
    themes::{Theme, get_current_theme_import_path, get_theme_by_name, list_themes},
};

/// Switches the active Alacritty theme by updating the config file.
pub fn switch_theme(name: &str, custom_theme_path: Option<&Path>) -> Result<Theme> {
    let theme = get_theme_by_name(name, custom_theme_path)?;
    let config_path = get_alacritty_config_path()?;

    if !config_path.exists() {
        return Err(AlthemerError::ConfigNotFound(config_path));
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
pub fn select_theme(custom_theme_path: Option<&Path>, config: &AlthemerConfig) -> Result<Theme> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(AlthemerError::NoTerminal);
    }

    let themes = list_themes(custom_theme_path)?;
    let current_path = get_current_theme_import_path().ok().flatten();
    let current = current_path
        .as_ref()
        .and_then(|p| themes.iter().find(|t| &t.path == p));

    match pick_theme(&themes, current, config) {
        Some(theme) => Ok(theme),
        None => Err(AlthemerError::InteractiveError(
            "No theme selected".to_string(),
        )),
    }
}
