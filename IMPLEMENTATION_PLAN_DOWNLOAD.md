# Althemer Theme Download Implementation Plan

## Overview

Add the ability to download Alacritty themes from GitHub repositories, with automatic fallback to the official `alacritty/alacritty-theme` repo when no URL is provided.

**Reference implementations:**
- `~/dev/codes/bob` - Rust download patterns with reqwest + indicatif
- `~/dev/codes/alacritty-theme-switch` - TypeScript GitHub client for listing/downloading themes

---

## 1. Dependencies (`Cargo.toml`)

Add async HTTP client and progress bar:

```toml
[dependencies]
# Existing dependencies...

# New dependencies for downloading
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["fs", "rt-multi-thread", "macros"] }
futures-util = "0.3"
indicatif = "0.18"
anyhow = "1.0"
```

**Reference:** `bob/Cargo.toml` and `bob/src/handlers/install_handler.rs` for streaming download patterns.

---

## 2. Error Handling (`src/error.rs`)

Add new error variants for download operations:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AlthemerError {
    // ... existing variants ...

    #[error("GitHub API error: {0}")]
    GitHubApi(String),

    #[error("Download failed: {0}")]
    Download(String),

    #[error("Failed to parse repository URL: {0}")]
    InvalidRepoUrl(String),

    #[error("GitHub rate limit exceeded")]
    RateLimitExceeded,

    #[error("Repository not found: {0}")]
    RepoNotFound(String),
}
```

---

## 3. GitHub Module (`src/github.rs`)

### Purpose
Handle GitHub API interactions for listing TOML files and constructing download URLs.

### Data Structures

```rust
/// Repository tree item from GitHub API
#[derive(Debug, Deserialize)]
pub struct TreeItem {
    pub path: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub sha: String,
    pub size: Option<u64>,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct TreeResponse {
    pub sha: String,
    pub url: String,
    pub tree: Vec<TreeItem>,
    pub truncated: bool,
}

/// Parsed repository info from URL
#[derive(Debug, Clone)]
pub struct RepoInfo {
    pub owner: String,
    pub repo: String,
}
```

### Key Functions

```rust
/// Parse GitHub URL to extract owner/repo
/// Supports:
///   - https://github.com/owner/repo
///   - https://github.com/owner/repo.git
///   - git@github.com:owner/repo.git
pub fn parse_github_url(url: &str) -> Option<RepoInfo>;

/// Fetch repository tree recursively via GitHub API
pub async fn list_repo_tree(
    client: &Client,
    owner: &str,
    repo: &str,
    branch: &str,
) -> Result<TreeResponse>;

/// Filter TOML files from tree response
pub fn filter_toml_files(tree: &[TreeItem]) -> Vec<&TreeItem>;

/// Build raw content URL for a file
pub fn build_raw_url(owner: &str, repo: &str, branch: &str, path: &str) -> String;
```

### GitHub API Patterns (from `bob/src/github_requests.rs`)

```rust
pub async fn make_github_request(
    client: &Client,
    url: impl AsRef<str> + reqwest::IntoUrl,
) -> Result<String> {
    let response = client
        .get(url)
        .header("user-agent", "althemer")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?
        .text()
        .await?;
    Ok(response)
}

pub fn deserialize_response<T: DeserializeOwned>(response: &str) -> Result<T> {
    let value: serde_json::Value = serde_json::from_str(response)?;
    // Check for error messages (rate limiting, not found, etc.)
    if let Some(msg) = value.get("message").and_then(|m| m.as_str()) {
        if msg.contains("rate limit") {
            return Err(AlthemerError::RateLimitExceeded);
        }
        return Err(AlthemerError::GitHubApi(msg.to_string()));
    }
    Ok(serde_json::from_value(value)?)
}
```

---

## 4. Download Module (`src/downloader.rs`)

### Purpose
Download theme files with progress bar display.

### Key Functions

```rust
use indicatif::{ProgressBar, ProgressStyle};

/// Download a single file with progress callback
pub async fn download_file(
    client: &Client,
    url: &str,
    dest: &Path,
    progress_callback: impl Fn(u64, u64),
) -> Result<u64>;

/// Download multiple files with overall progress bar
/// Pattern from alacritty-theme-switch/src/commands/download-themes.ts
pub async fn download_themes<F>(
    client: &Client,
    files: &[DownloadFile],
    output_dir: &Path,
    on_progress: F,
) -> Result<Vec<PathBuf>>
where
    F: Fn(usize, usize); // (current, total)
```

### Progress Bar Style (from `bob/install_handler.rs`)

```rust
ProgressStyle::with_template(
    "{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
)
.progress_chars("█  ")
```

### Streaming Download Pattern (from `bob`)

```rust
let response = client.get(url).header("user-agent", "bob").send().await?;
let total_size = response.content_length().unwrap_or(0);
let pb = ProgressBar::new(total_size);

let mut file = tokio::fs::File::create(dest).await?;
let mut stream = response.bytes_stream();

while let Some(chunk) = stream.next().await {
    let bytes = chunk?;
    tokio::io::AsyncWriteExt::write_all(&mut file, &bytes).await?;
    pb.set_position(file.metadata().await?.len());
}
```

---

## 5. Remote Module (`src/remote.rs`)

### Purpose
High-level interface combining github + downloader with fallback logic.

### Constants

```rust
/// Default repository when no URL provided
const DEFAULT_THEME_REPO: &str = "alacritty/alacritty-theme";
const DEFAULT_BRANCH: &str = "master";
```

### Key Functions

```rust
use crate::github::{parse_github_url, RepoInfo};
use crate::downloader::download_themes;

/// Remote download options
pub struct DownloadOptions {
    pub url: Option<String>,     // GitHub URL or owner/repo
    pub branch: Option<String>,   // Defaults to "master"
    pub force: bool,              // Overwrite existing
}

/// Download themes from GitHub repository
///
/// Logic:
/// 1. Parse URL or use DEFAULT_THEME_REPO
/// 2. Fetch repository tree via GitHub API
/// 3. Filter for .toml files
/// 4. Download each theme with progress bar
/// 5. Return list of downloaded theme paths
pub async fn download_remote_themes(
    client: &Client,
    themes_dir: &Path,
    options: DownloadOptions,
) -> Result<Vec<String>>;

/// Extract owner/repo from URL or use as-is
fn resolve_repo(url: Option<&str>) -> (String, Option<String>) {
    match url {
        Some(u) => {
            if let Some(info) = parse_github_url(u) {
                (format!("{}/{}", info.owner, info.repo), Some(info.owner))
            } else {
                // Assume it's already owner/repo format
                (u.to_string(), None)
            }
        }
        None => (DEFAULT_THEME_REPO.to_string(), None),
    }
}
```

### Fallback Logic

```rust
/// Main download flow with fallback
pub async fn download_with_fallback(
    client: &Client,
    themes_dir: &Path,
    url: Option<&str>,
    branch: Option<&str>,
) -> Result<Vec<String>> {
    let (repo, _owner) = resolve_repo(url);
    let branch = branch.unwrap_or(DEFAULT_BRANCH);

    // Try downloading from specified repo
    let result = download_remote_themes(client, themes_dir, &DownloadOptions {
        url: Some(repo.clone()),
        branch: Some(branch.to_string()),
        force: false,
    }).await;

    match result {
        Ok(themes) => Ok(themes),
        Err(e) => {
            // If no URL was specified, don't retry (we already used default)
            if url.is_some() {
                // Could implement retry logic here
                Err(e)
            } else {
                Err(e)
            }
        }
    }
}
```

---

## 6. CLI Extension (`src/cli.rs`)

### Add New Command

```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands ...

    /// Download themes from a GitHub repository
    Download {
        /// GitHub repository URL or owner/repo format
        /// [default: alacritty/alacritty-theme]
        #[arg(short, long)]
        url: Option<String>,

        /// Repository in owner/repo format (alternative to --url)
        #[arg()]
        repo: Option<String>,

        /// Branch to download from [default: master]
        #[arg(short, long)]
        branch: Option<String>,

        /// Overwrite existing theme files
        #[arg(short, long)]
        force: bool,
    },
}
```

---

## 7. Main Integration (`src/main.rs`)

### Wire Up Download Command

```rust
use remote::download_remote_themes;
use std::path::PathBuf;

fn main() {
    // ... existing setup ...
    
    let client = reqwest::Client::new();

    match cli.command {
        // ... existing commands ...

        Some(Commands::Download { url, repo, branch, force }) => {
            let effective_url = url.or(repo);
            let themes_path = themes_path
                .map(PathBuf::from)
                .or_else(|| config.themes_dir.clone())
                .ok_or_else(|| AlthemerError::ConfigurationError(
                    "Themes directory not configured".to_string()
                ))?;

            let options = remote::DownloadOptions {
                url: effective_url,
                branch,
                force,
            };

            let downloaded = tokio::runtime::Runtime::new()?
                .block_on(download_remote_themes(&client, &themes_path, options))?;

            println!("✓ Downloaded {} themes to {}", 
                     downloaded.len(), 
                     themes_path.display());
        }
    }
}
```

### Module Declaration

```rust
mod github;
mod downloader;
mod remote;
```

---

## 8. File Structure

```
althemer/
├── src/
│   ├── main.rs           # Update: add download command
│   ├── cli.rs            # Update: add Download command variant
│   ├── error.rs          # Update: add download error types
│   ├── github.rs         # NEW: GitHub API client
│   ├── downloader.rs     # NEW: Progress bar downloads
│   ├── remote.rs         # NEW: High-level interface
│   ├── themes.rs         # (existing)
│   ├── alacritty.rs      # (existing)
│   ├── config.rs         # (existing)
│   ├── switcher.rs       # (existing)
│   ├── picker.rs         # (existing)
│   └── tui.rs            # (existing)
└── Cargo.toml            # Update: add dependencies
```

---

## 9. Implementation Phases

### Phase 1: Foundation
1. Add `reqwest`, `tokio`, `futures-util`, `indicatif`, `anyhow` to `Cargo.toml`
2. Create `src/error.rs` with download error variants
3. Create `src/github.rs` with URL parsing and API client
4. Test GitHub API calls manually

### Phase 2: Core Download
1. Create `src/downloader.rs` with progress bar
2. Implement `download_file()` with streaming
3. Implement `download_themes()` batch download
4. Test with single file download

### Phase 3: Remote Interface
1. Create `src/remote.rs` with high-level API
2. Implement fallback to `alacritty/alacritty-theme`
3. Wire up CLI command
4. End-to-end test

### Phase 4: Polish
1. Add rate limit handling with user-friendly message
2. Handle network errors gracefully
3. Add total progress bar for batch downloads

---

## 10. Key Patterns from Reference Code

### URL Parsing (from `alacritty-theme-switch`)

```rust
fn parse_github_url(url: &str) -> Option<RepoInfo> {
    // HTTPS: https://github.com/owner/repo
    // SSH: git@github.com:owner/repo
    // Short: owner/repo
}
```

### GitHub Request (from `bob`)

```rust
async fn make_github_request<T: AsRef<str> + reqwest::IntoUrl>(
    client: &Client,
    url: T,
) -> Result<String> {
    client
        .get(url)
        .header("user-agent", "althemer")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?
        .text()
        .await
}
```

### Progress Bar (from `bob`)

```rust
let pb = ProgressBar::new(total_bytes);
pb.set_style(
    ProgressStyle::with_template(
        "{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
    )
    .progress_chars("█  ")
);
```

### Streaming Download (from `bob`)

```rust
let response = client.get(url).send().await?;
let total_size = response.content_length().unwrap_or(0);
let mut stream = response.bytes_stream();

while let Some(chunk) = stream.next().await {
    let bytes = chunk?;
    file.write_all(&bytes).await?;
}
```

---

## 11. Testing Checklist

- [ ] Parse GitHub HTTPS URL
- [ ] Parse GitHub SSH URL  
- [ ] Parse owner/repo shorthand
- [ ] List TOML files from repo tree
- [ ] Download single theme file
- [ ] Download multiple themes with progress
- [ ] Fallback to default repo when no URL
- [ ] Handle rate limit error
- [ ] Handle 404 not found
- [ ] Overwrite existing with --force
- [ ] Create themes directory if missing

---

## 12. Usage Examples

```bash
# Download from default repo (alacritty/alacritty-theme)
althemer download

# Download from specific repo
althemer download --url https://github.com/catppuccin/alacritty
althemer download catppuccin/alacritty

# Download from specific branch
althemer download --url https://github.com/user/themes --branch develop

# Overwrite existing themes
althemer download --force
```
