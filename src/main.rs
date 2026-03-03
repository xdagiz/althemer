use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod config;
mod error;
mod switcher;
mod themes;

use crate::switcher::switch_theme;
use crate::themes::{get_current_theme, list_themes};

#[derive(Parser)]
#[command(name = "althemer")]
#[command(about = "A cli to switch b/n alacritty themes", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Custom themes directory
    #[arg(long, global = true)]
    pub themes: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
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

fn main() {
    let cli = Cli::parse();

    let custom_themes_path = cli.themes.as_deref();

    match &cli.command {
        Commands::List => {
            let themes = list_themes(custom_themes_path).expect("Failed to list themes");
            println!("Available themes ({} total):", themes.len());
            for theme in themes {
                println!("  - {}", theme.name);
            }
        }
        Commands::Current => match get_current_theme(custom_themes_path) {
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
        Commands::Switch { theme } => match switch_theme(theme, custom_themes_path) {
            Ok(theme) => {
                println!("✓ Switched to theme: {}", theme.name);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
    }
}
