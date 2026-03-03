use std::path::Path;

use crate::{
    config::{get_alacritty_config_path, read_config, write_config},
    error::Result,
    themes::{Theme, get_theme_by_name},
};

/// Switches the active Alacritty theme by updating the config file.
pub fn switch_theme(name: &str, custom_theme_path: Option<&Path>) -> Result<Theme> {
    let theme = get_theme_by_name(name, custom_theme_path)?;
    let config_path = get_alacritty_config_path()?;

    if !config_path.exists() {
        return Err(crate::error::AppError::ConfigNotFound(config_path));
    }

    let mut config = read_config(&config_path)?;
    let theme_path_str = theme.path.to_string_lossy().to_string();
    config.general.import = vec![theme_path_str];

    write_config(&config_path, &config)?;

    Ok(theme)
}
