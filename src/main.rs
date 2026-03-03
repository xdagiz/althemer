use cli::Cli;

mod cli;
mod config;
mod error;
mod switcher;
mod themes;

fn main() {
    Cli::parse_args();
}
