use crate::{
    alacritty::get_themes_dir,
    cli::Cli,
    error::{AlthemerError, Result},
};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AlthemerConfig {
    #[serde(default = "default_themes_dir")]
    pub themes_dir: Option<PathBuf>,

    #[serde(default = "default_show_preview")]
    pub show_preview: bool,

    #[serde(default = "default_quit_on_select")]
    pub quit_on_select: bool,

    #[serde(default = "default_picker_reversed")]
    pub picker_reversed: bool,

    #[serde(default = "default_picker_sort_results")]
    pub picker_sort_results: bool,

    #[serde(skip)]
    pub config_path: Option<PathBuf>,
}

impl Default for AlthemerConfig {
    fn default() -> Self {
        Self {
            themes_dir: default_themes_dir(),
            show_preview: default_show_preview(),
            quit_on_select: default_quit_on_select(),
            picker_reversed: default_picker_reversed(),
            picker_sort_results: default_picker_sort_results(),
            config_path: None,
        }
    }
}

fn default_show_preview() -> bool {
    true
}

fn default_quit_on_select() -> bool {
    false
}

fn default_picker_reversed() -> bool {
    false
}

fn default_picker_sort_results() -> bool {
    true
}

fn default_themes_dir() -> Option<PathBuf> {
    if let Ok(p) = get_themes_dir(None) {
        return Some(p);
    }

    None
}

pub fn get_config_path() -> Option<PathBuf> {
    config_dir().map(|path| path.join("althemer").join("config.json"))
}

impl AlthemerConfig {
    pub fn new(cli: &Cli) -> Result<Self> {
        let mut config = if let Some(config_path) = &cli.config {
            AlthemerConfig::from_file(config_path)?
        } else {
            match get_config_path() {
                Some(p) if p.exists() => AlthemerConfig::from_file(&p)?,
                Some(p) => {
                    let config = AlthemerConfig {
                        config_path: Some(p),
                        ..Default::default()
                    };
                    config.save()?;
                    config
                }
                None => AlthemerConfig::default(),
            }
        };

        if config.config_path.is_none() {
            if let Some(config_path) = cli.config.as_deref() {
                config.config_path = Some(config_path.to_path_buf());
            } else {
                config.config_path = get_config_path();
            }
        }

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let fallback_path = get_config_path();
        let config_path = self
            .config_path
            .as_deref()
            .or(fallback_path.as_deref())
            .ok_or_else(|| {
                AlthemerError::ConfigurationError(
                    "Could not determine where to save configuration".to_string(),
                )
            })?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;

        Ok(())
    }

    pub fn from_file(file_path: &Path) -> Result<Self> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).map_err(|e| {
            AlthemerError::ConfigurationError(format!(
                "Failed to parse config at '{}': {}",
                file_path.display(),
                e
            ))
        })
    }
}
