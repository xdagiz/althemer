use crate::{
    alacritty::{get_alacritty_config_path, get_themes_dir},
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
    #[serde(default)]
    pub themes_dir: Option<PathBuf>,

    #[serde(default)]
    pub alacritty_config: Option<PathBuf>,

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
            alacritty_config: None,
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
    let alacritty_config = get_alacritty_config_path(None, None).ok()?;

    if let Ok(p) = get_themes_dir(None, &alacritty_config) {
        return Some(p);
    }

    None
}

/// Environment variable holding the XDG base directory for user configuration.
pub const XDG_CONFIG_HOME_ENV: &str = "XDG_CONFIG_HOME";

fn althemer_config_file(config_home: &Path) -> PathBuf {
    config_home.join("althemer").join("config.json")
}

/// Resolves althemer's own config file, in order of preference:
///
/// 1. `$XDG_CONFIG_HOME/althemer/config.json`
/// 2. `~/.config/althemer/config.json`
/// 3. `<platform config dir>/althemer/config.json`
///
/// The first candidate that already exists wins, so existing installs (notably
/// on macOS, where the platform config dir is `~/Library/Application Support`)
/// keep working. When none exists, the highest-priority candidate is returned
/// as the location to create — matching the documented default of
/// `~/.config/althemer/config.json`.
fn resolve_config_path(
    xdg_config_home: Option<&Path>,
    home_dir: Option<&Path>,
    platform_config_dir: Option<&Path>,
    exists: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let candidates: Vec<PathBuf> = [
        xdg_config_home.map(althemer_config_file),
        home_dir.map(|home| althemer_config_file(&home.join(".config"))),
        platform_config_dir.map(althemer_config_file),
    ]
    .into_iter()
    .flatten()
    .collect();

    candidates
        .iter()
        .find(|path| exists(path))
        .cloned()
        .or_else(|| candidates.into_iter().next())
}

/// Same as [`resolve_config_path`], reading the environment and filesystem itself.
fn xdg_config_home(var: Option<std::ffi::OsString>) -> Option<PathBuf> {
    var.filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}

pub fn get_config_path() -> Option<PathBuf> {
    let xdg_config_home = xdg_config_home(std::env::var_os(XDG_CONFIG_HOME_ENV));

    resolve_config_path(
        xdg_config_home.as_deref(),
        dirs::home_dir().as_deref(),
        config_dir().as_deref(),
        |path| path.exists(),
    )
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

        // Themes live next to the alacritty config unless configured otherwise.
        if config.themes_dir.is_none() {
            let alacritty_config = get_alacritty_config_path(
                cli.alacritty_config.as_deref(),
                config.alacritty_config.as_deref(),
            )?;
            config.themes_dir = get_themes_dir(None, &alacritty_config).ok();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Builds an exists-predicate over a fixed set of existing paths.
    fn exists_in(paths: &[PathBuf]) -> impl Fn(&Path) -> bool + use<> {
        let existing: HashSet<PathBuf> = paths.iter().cloned().collect();
        move |path: &Path| existing.contains(path)
    }

    const XDG: &str = "/xdg";
    const HOME: &str = "/home/user";
    const PLATFORM: &str = "/home/user/Library/Application Support";

    fn xdg_file() -> PathBuf {
        PathBuf::from(XDG).join("althemer").join("config.json")
    }

    fn dot_config_file() -> PathBuf {
        PathBuf::from(HOME)
            .join(".config")
            .join("althemer")
            .join("config.json")
    }

    fn platform_file() -> PathBuf {
        PathBuf::from(PLATFORM).join("althemer").join("config.json")
    }

    #[test]
    fn xdg_config_home_wins_when_it_exists() {
        let path = resolve_config_path(
            Some(Path::new(XDG)),
            Some(Path::new(HOME)),
            Some(Path::new(PLATFORM)),
            exists_in(&[xdg_file(), dot_config_file(), platform_file()]),
        )
        .unwrap();
        assert_eq!(path, xdg_file());
    }

    #[test]
    fn dot_config_is_used_when_xdg_is_unset() {
        let path = resolve_config_path(
            None,
            Some(Path::new(HOME)),
            Some(Path::new(PLATFORM)),
            exists_in(&[dot_config_file(), platform_file()]),
        )
        .unwrap();
        assert_eq!(path, dot_config_file());
    }

    #[test]
    fn dot_config_is_used_when_the_xdg_candidate_does_not_exist() {
        let path = resolve_config_path(
            Some(Path::new(XDG)),
            Some(Path::new(HOME)),
            Some(Path::new(PLATFORM)),
            exists_in(&[dot_config_file()]),
        )
        .unwrap();
        assert_eq!(path, dot_config_file());
    }

    #[test]
    fn falls_back_to_the_platform_config_dir_for_existing_installs() {
        let path = resolve_config_path(
            Some(Path::new(XDG)),
            Some(Path::new(HOME)),
            Some(Path::new(PLATFORM)),
            exists_in(&[platform_file()]),
        )
        .unwrap();

        assert_eq!(path, platform_file());
    }

    #[test]
    fn creates_in_xdg_config_home_when_nothing_exists() {
        let path = resolve_config_path(
            Some(Path::new(XDG)),
            Some(Path::new(HOME)),
            Some(Path::new(PLATFORM)),
            exists_in(&[]),
        )
        .unwrap();

        assert_eq!(path, xdg_file());
    }

    #[test]
    fn creates_in_dot_config_when_nothing_exists_and_xdg_is_unset() {
        let path = resolve_config_path(
            None,
            Some(Path::new(HOME)),
            Some(Path::new(PLATFORM)),
            exists_in(&[]),
        )
        .unwrap();

        assert_eq!(path, dot_config_file());
    }

    #[test]
    fn falls_back_to_the_platform_config_dir_without_a_home_dir() {
        let path =
            resolve_config_path(None, None, Some(Path::new(PLATFORM)), exists_in(&[])).unwrap();

        assert_eq!(path, platform_file());
    }

    #[test]
    fn resolves_to_nothing_without_any_candidate() {
        assert!(resolve_config_path(None, None, None, exists_in(&[])).is_none());
    }

    #[test]
    fn relative_xdg_config_home_is_ignored_for_existing_files() {
        let xdg = xdg_config_home(Some(std::ffi::OsString::from("relative/path")));
        let path = resolve_config_path(
            xdg.as_deref(),
            Some(Path::new(HOME)),
            Some(Path::new(PLATFORM)),
            exists_in(&[dot_config_file()]),
        )
        .unwrap();
        assert_eq!(path, dot_config_file());
    }

    #[test]
    fn relative_xdg_config_home_is_ignored_for_creation_target() {
        let xdg = xdg_config_home(Some(std::ffi::OsString::from("relative/path")));
        let path = resolve_config_path(
            xdg.as_deref(),
            Some(Path::new(HOME)),
            Some(Path::new(PLATFORM)),
            exists_in(&[]),
        )
        .unwrap();
        assert_eq!(path, dot_config_file());
    }

    #[test]
    fn absolute_xdg_config_home_is_kept() {
        let xdg = xdg_config_home(Some(std::ffi::OsString::from(XDG)));
        assert_eq!(xdg, Some(xdg_file()));
    }

    #[test]
    fn empty_xdg_config_home_is_ignored() {
        let xdg = xdg_config_home(Some(std::ffi::OsString::from("")));
        assert!(xdg.is_none());
    }

    #[test]
    fn existing_file_on_disk_is_detected() {
        let dir = tempfile::tempdir().unwrap();
        let config_file = althemer_config_file(dir.path());

        std::fs::create_dir_all(config_file.parent().unwrap()).unwrap();
        std::fs::write(&config_file, "{}").unwrap();

        let path = resolve_config_path(
            None,
            Some(Path::new(HOME)),
            Some(dir.path()),
            |path: &Path| path.exists(),
        )
        .unwrap();
        assert_eq!(path, config_file);
    }
}
