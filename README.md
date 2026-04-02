# Althemer

A CLI and interactive TUI to switch between Alacritty themes with fuzzy search.

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)

## Features

- **Interactive TUI** - Visually browse and select themes with fuzzy search
- **List themes** - View all available themes in your themes directory
- **Check current** - See which theme is currently active
- **Quick switch** - Switch to any theme by name from the CLI
- **Configurable** - Set your themes directory and preferences

## Installation

### From source

```bash
git clone https://github.com/xdagiz/althemer
cd althemer
cargo install --locked --path .
```

### Requirements

- Rust
- Alacritty
- A themes directory with `.toml` alacritty theme files

## Usage

### Interactive mode (TUI)

Launch the interactive theme picker:

```bash
althemer
```

Use arrow keys to navigate, type to fuzzy search, and press Enter to apply.

### List all themes and quick switch

```bash
althemer list
```

### Check current theme

```bash
althemer current
```

### Switch to a theme by name

```bash
althemer switch <theme-name>
```

### Download themes from GitHub

```bash
# download from the default github repo [alacritty/alacritty-theme]
althemer download
# download from a specific repo
althemer download https://github.com/user/repo # or just user/repo
# use -b to specify the branch
althemer download https://github.com/user/repo -b develop
# -f will overwrite existing themes
althemer download -f
```

### Configure althemer

```bash
althemer configure
```

## Configuration

Althemer looks for themes in:

1. `--themes` CLI argument
2. `themes_dir` in config file (default: `~/.config/althemer/config.json`)

### Config file location

Default: `~/.config/althemer/config.json`
```json
{
  "themes_dir": "/home/xdagiz/.config/alacritty/themes",
  "show_preview": true,
  "quit_on_select": false,
  "picker_reversed": false,
  "picker_sort_results": true
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

___
By [xdagiz](https://github.com/xdagiz)
