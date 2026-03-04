use std::io;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod config;
mod error;
mod picker;
mod switcher;
mod themes;
mod tui;

use crate::error::AppError;
use crate::switcher::{select_theme, switch_theme};
use crate::themes::{get_current_theme, list_themes};
use crate::tui::App;

#[derive(Parser)]
#[command(name = "althemer")]
#[command(about = "A cli to switch b/n alacritty themes", long_about = None)]
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

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let custom_themes_path = cli.themes.as_deref();

    if cli.command.is_none() {
        return ratatui::run(|term| App::new(custom_themes_path).run(term));
    }

    match &cli.command {
        Some(Commands::List) => {
            match select_theme(custom_themes_path) {
                Ok(_) => {}
                Err(AppError::NoTerminal) => {
                    // Not running in terminal, show plain list
                    match list_themes(custom_themes_path) {
                        Ok(themes) => {
                            println!("Available themes ({} total):", themes.len());
                            for theme in themes {
                                println!("  - {}", theme.name);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error listing themes: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Current) => match get_current_theme(custom_themes_path) {
            Ok(Some(theme)) => {
                println!("Current theme: {}", theme.name);
            }
            Ok(None) => {
                println!("No theme currently imported");
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::exit(1);
            }
        },
        Some(Commands::Switch { theme }) => match switch_theme(theme, custom_themes_path) {
            Ok(theme) => {
                println!("✓ Switched to theme: {}", theme.name);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        None => unreachable!(),
    }

    Ok(())
}
