use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod config;
mod error;
mod picker;
mod switcher;
mod themes;
mod tui;

use crate::error::{AppError, Result};
use crate::switcher::{select_theme, switch_theme};
use crate::themes::{get_current_theme, list_themes};
use crate::tui::App;

#[derive(Parser)]
#[command(name = "althemer")]
#[command(about = "A cli & tui to switch b/n alacritty themes", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Custom themes directory
    #[arg(long, global = true)]
    pub themes: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    List,
    Current,
    Switch {
        #[arg()]
        theme: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let custom_themes_path = cli.themes.as_deref();

    if cli.command.is_none() {
        return ratatui::run(|term| App::new(custom_themes_path).run(term))
            .map_err(|e| AppError::InteractiveError(e.to_string()));
    }

    match &cli.command {
        Some(Commands::List) => match select_theme(custom_themes_path) {
            Ok(_) => {}
            Err(AppError::NoTerminal) => {
                let themes = list_themes(custom_themes_path)?;
                println!("Available themes ({} total):", themes.len());
                for theme in themes {
                    println!("  - {}", theme.name);
                }
            }
            Err(e) => return Err(e),
        },
        Some(Commands::Current) => match get_current_theme(custom_themes_path) {
            Ok(Some(theme)) => {
                println!("Current theme: {}", theme.name);
            }
            Ok(None) => {
                println!("No theme currently imported");
            }
            Err(err) => return Err(err),
        },
        Some(Commands::Switch { theme }) => {
            let theme = switch_theme(theme, custom_themes_path)?;
            println!("✓ Switched to theme: {}", theme.name);
        }
        None => unreachable!(),
    }

    Ok(())
}
