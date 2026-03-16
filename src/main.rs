mod config;
mod error;
mod picker;
mod switcher;
mod themes;
mod tui;

use std::path::PathBuf;

use clap::Parser;
use config::{AlthemerConfig, Cli, Commands};
use error::{AlthemerError, Result};
use inquire::Text;
use switcher::{select_theme, switch_theme};
use themes::{get_current_theme, list_themes};
use tui::App;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut config = AlthemerConfig::load(cli.config.as_deref())?;
    let themes_path = cli.themes.as_deref().or(config.themes_dir.as_deref());
    if cli.command.is_none() {
        return ratatui::run(|term| App::new(themes_path, &config).run(term))
            .map_err(|e| AlthemerError::InteractiveError(e.to_string()));
    }

    match &cli.command {
        Some(Commands::List) => match select_theme(themes_path) {
            Ok(t) => {
                let theme = switch_theme(&t.name, themes_path)?;
                println!("✓ Switched to theme: {}", theme.name);
            }
            Err(AlthemerError::NoTerminal) => {
                let themes = list_themes(themes_path)?;
                println!("Available themes ({} total):", themes.len());
                for theme in themes {
                    println!("  - {}", theme.name);
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
            let theme = switch_theme(theme, themes_path)?;
            println!("✓ Switched to theme: {}", theme.name);
        }
        Some(Commands::Configure) => {
            let themes_dir = Text::new("Enter path to themes dir:")
                .prompt()
                .map_err(|e| AlthemerError::ConfigurationError(e.to_string()))?;
            if !themes_dir.is_empty() {
                config.themes_dir = Some(PathBuf::from(&themes_dir));
                config.save()?;
                println!("✓ Successfully configured!");
            } else {
                println!("Nothing changed!");
            }
        }
        None => unreachable!(),
    }

    Ok(())
}
