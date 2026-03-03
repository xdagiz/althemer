use crate::{
    switcher::switch_theme,
    themes::{get_current_theme, list_themes},
};
use clap::{Parser, Subcommand};

mod config;
mod error;
mod switcher;
mod themes;

#[derive(Parser)]
#[command(name = "althemer")]
#[command(about = "A cli to switch b/n alacritty themes", long_about = None)]
#[command(version)]
pub struct Cli {
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

    match &cli.command {
        Commands::List => {
            let themes = list_themes().expect("Failed to list themes");
            println!("Available themes ({} total):", themes.len());
            for theme in themes {
                println!("  - {}", theme.name);
            }
        }
        Commands::Current => match get_current_theme() {
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

        Commands::Switch { theme } => match switch_theme(theme) {
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
