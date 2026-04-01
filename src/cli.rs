use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "althemer")]
#[command(about = "A cli & tui to switch b/n alacritty themes", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Path to althemer config file [default: ~/.config/althemer/config.json]
    #[arg(long, short, global = true)]
    pub config: Option<PathBuf>,

    /// Custom themes directory [default: ~/.config/alacritty/themes]
    #[arg(long, short, global = true)]
    pub themes: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Download themes from a github repo
    Download {
        #[arg()]
        repo: Option<String>,
        #[arg(short = 'b', long)]
        branch: Option<String>,
        #[arg(short = 'f', long)]
        force: bool,
    },
    /// List available themes
    List,
    /// Get the current theme
    Current,
    /// Switch to a theme
    Switch {
        #[arg()]
        theme: String,
    },
    /// Configure althemer
    Configure,
}
