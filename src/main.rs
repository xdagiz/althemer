use std::{path::PathBuf, process};

use clap::Parser;
use config::{AlthemerConfig, Cli, Commands};
use error::{AlthemerError, Result};
use inquire::{Confirm, Text};
use switcher::{select_theme, switch_theme};
use themes::{get_current_theme, list_themes};
use tui::App;

mod config;
mod error;
mod picker;
mod switcher;
mod themes;
mod tui;

fn main() {
    if let Err(e) = run() {
        eprintln!("\x1b[91m\rerror:\x1b[0m {e}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let config = AlthemerConfig::new(&cli)?;
    let themes_path = cli.themes.as_deref().or(config.themes_dir.as_deref());

    match cli.command {
        // directly run the tui if no args are passed
        None => ratatui::run(|term| App::new(themes_path, &config).run(term))
            .map_err(|e| AlthemerError::InteractiveError(e.to_string()))?,
        // handle subcommands
        Some(Commands::List) => match select_theme(themes_path, &config) {
            Ok(t) => {
                let theme = switch_theme(&t.name, themes_path)?;
                println!("✓ Switched to theme: {}", theme.name);
            }
            Err(AlthemerError::NoTerminal) => {
                let themes = list_themes(themes_path)?;
                for theme in themes {
                    println!("{}", theme.name);
                }
            }
            Err(e) => return Err(e),
        },
        Some(Commands::Current) => match get_current_theme(themes_path) {
            Ok(Some(theme)) => {
                println!("Current theme: {}", theme.name);
            }
            Ok(None) => {
                println!("No theme currently imported");
            }
            Err(err) => return Err(err),
        },
        Some(Commands::Switch { theme }) => {
            let theme = switch_theme(&theme, themes_path)?;
            println!("✓ Switched to theme: {}", theme.name);
        }
        Some(Commands::Configure) => {
            let mut config = config;

            let themes_dir = Text::new("Enter path to themes dir (leave empty to set default):")
                .prompt()
                .map_err(|e| AlthemerError::ConfigurationError(e.to_string()))?;

            if themes_dir.is_empty() && config.themes_dir.is_none() {
                if let Some(path) = dirs::config_dir() {
                    config.themes_dir = Some(path.join("alacritty").join("themes"));
                }
            } else if !themes_dir.is_empty() {
                config.themes_dir = Some(PathBuf::from(&themes_dir));
            }

            let show_preview = Confirm::new("Enable theme preview?")
                .with_default(config.show_preview)
                .prompt()
                .map_err(|e| AlthemerError::ConfigurationError(e.to_string()))?;

            let quit_on_select = Confirm::new("Quit after applying a theme?")
                .with_default(config.quit_on_select)
                .prompt()
                .map_err(|e| AlthemerError::ConfigurationError(e.to_string()))?;

            config.show_preview = show_preview;
            config.quit_on_select = quit_on_select;
            config.save()?;

            println!("✓ Successfully configured!");
        }
    }

    Ok(())
}
