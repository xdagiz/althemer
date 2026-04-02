use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::shells;
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
    Switch { theme: String },
    /// Configure althemer
    Configure,
    /// Generate shell completion
    Completion { shell: Shell },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
#[allow(clippy::enum_variant_names)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

impl clap_complete::Generator for Shell {
    fn file_name(&self, name: &str) -> String {
        match self {
            Shell::Bash => shells::Bash.file_name(name),
            Shell::Elvish => shells::Elvish.file_name(name),
            Shell::Fish => shells::Fish.file_name(name),
            Shell::PowerShell => shells::PowerShell.file_name(name),
            Shell::Zsh => shells::Zsh.file_name(name),
        }
    }

    fn generate(&self, cmd: &clap::Command, buf: &mut dyn std::io::Write) {
        self.try_generate(cmd, buf)
            .expect("failed to write completion file");
    }

    fn try_generate(
        &self,
        cmd: &clap::Command,
        buf: &mut dyn std::io::Write,
    ) -> Result<(), std::io::Error> {
        match self {
            Shell::Bash => shells::Bash.try_generate(cmd, buf),
            Shell::Elvish => shells::Elvish.try_generate(cmd, buf),
            Shell::Fish => shells::Fish.try_generate(cmd, buf),
            Shell::PowerShell => shells::PowerShell.try_generate(cmd, buf),
            Shell::Zsh => shells::Zsh.try_generate(cmd, buf),
        }
    }
}
