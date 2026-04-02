# Althemer

A CLI and interactive TUI to switch between Alacritty themes with fuzzy search.

[![Crates.io](https://img.shields.io/crates/v/althemer.svg)](https://crates.io/crates/althemer)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

https://github.com/user-attachments/assets/41d92a1c-47bb-4ffd-8cd4-fb5d7e4eccb9

## Features

- **Interactive TUI** - Visually browse and select themes with live color preview
- **Dark/Light tabs** - Filter themes by category with tabbed navigation
- **Fuzzy search** - Type to filter themes in real-time with fuzzy matching
- **List themes** - View all available themes in your themes directory
- **Check current** - See which theme is currently active
- **Quick switch** - Switch to any theme by name from the CLI
- **Download themes** - Fetch theme collections directly from GitHub
- **Configurable** - Set your themes directory, preview preferences, and more

## Installation

### Cargo

```bash
cargo install althemer
```

### Cargo Binstall

```bash
cargo binstall althemer
```

### Pre-built binaries

Download the latest release binary for your platform from the [Releases](https://github.com/xdagiz/althemer/releases) page.

### From source (requires rust)

```bash
git clone https://github.com/xdagiz/althemer
cd althemer
cargo install --locked --path .
```

### Requirements

- Alacritty
- A themes directory with `.toml` alacritty theme files

## Usage

### Interactive mode (TUI)

Launch the interactive theme picker:

```bash
althemer
```

### Global CLI flags

```bash
althemer -t /path/to/themes    # Custom themes directory
althemer -c /path/to/config    # Custom config file location
```

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
## Configuration

Althemer looks for themes in:

1. `--themes` / `-t` CLI argument
2. `themes_dir` in config file (default: `~/.config/alacritty/themes`)

### Config file

Default location: `~/.config/althemer/config.json`

```json
{
  "themes_dir": "/home/xdagiz/.config/alacritty/themes",
  "show_preview": true,
  "quit_on_select": false,
  "picker_reversed": false,
  "picker_sort_results": true
}
```

### Config options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `themes_dir` | string | `~/.config/alacritty/themes` | Path to directory containing `.toml` theme files |
| `show_preview` | bool | `true` | Show color palette preview in TUI |
| `quit_on_select` | bool | `false` | Exit TUI after applying a theme |
| `picker_reversed` | bool | `false` | Reverse the picker display order |
| `picker_sort_results` | bool | `true` | Sort fuzzy search results by relevance |

You can interactively configure these options by running:

```bash
althemer configure
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

___
By [xdagiz](https://github.com/xdagiz)
