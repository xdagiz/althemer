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
    List,
    Current,
    Switch {
        #[arg()]
        theme: String,
    },
    Configure,
}
