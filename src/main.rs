mod config;
mod error;
mod picker;
mod switcher;
mod themes;
mod tui;

use clap::Parser;
use config::{AlthemerConfig, Cli, Commands};
use error::{AppError, Result};
use switcher::{select_theme, switch_theme};
use themes::{get_current_theme, list_themes};
use tui::App;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = AlthemerConfig::load(cli.config.as_deref())?;
    let themes_path = cli.themes.as_deref().or(config.themes_dir.as_deref());
    if cli.command.is_none() {
        return ratatui::run(|term| App::new(themes_path).run(term))
            .map_err(|e| AppError::InteractiveError(e.to_string()));
    }

    match &cli.command {
        Some(Commands::List) => match select_theme(themes_path) {
            Ok(_) => {}
            Err(AppError::NoTerminal) => {
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
        None => unreachable!(),
    }

    Ok(())
}
