use crate::{
    config::{get_alacritty_config_path, read_config, write_config},
    error::{AppError, Result},
    themes::{Theme, get_theme_by_name},
};

pub fn switch_theme(name: &str) -> Result<Theme> {
    let theme = get_theme_by_name(name)?;
    let config_path = get_alacritty_config_path()?;

    if !config_path.exists() {
        return Err(AppError::ConfigNotFound(config_path));
    }

    let mut config = read_config(&config_path)?;
    let theme_path_str = theme.path.to_string_lossy().to_string();
    config.general.import = vec![theme_path_str];

    write_config(&config_path, &config)?;

    Ok(theme)
}
