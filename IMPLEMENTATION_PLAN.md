# Althemer GitHub Theme Download Implementation Plan

## Overview

Add the ability to fetch, search, and download Alacritty themes directly from GitHub repositories, with a progress bar showing download status.

---

## 1. Dependency Additions

### Add to `Cargo.toml`

```toml
[dependencies]
# Existing dependencies...

# New dependencies for GitHub fetching and progress
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio = { version = "1", features = ["fs", "io-util", "rt-multi-thread", "macros"] }
futures-util = "0.3"
indicatif = "0.18"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

---

## 2. New Module: `src/github.rs`

### Purpose
Handle all GitHub API interactions for theme repositories.

### Data Structures

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// GitHub repository info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub full_name: String,
    pub description: Option<String>,
    pub stargazers_count: u32,
    pub topics: Vec<String>,
}

/// GitHub release or tag for theme archives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeRelease {
    pub tag_name: String,
    pub assets: Vec<ThemeAsset>,
}

/// Asset (downloadable file) in a release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// GitHub search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub total_count: u32,
    pub items: Vec<SearchItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchItem {
    pub full_name: String,
    pub description: Option<String>,
    pub stargazers_count: u32,
    pub topics: Vec<String>,
}
```

### Key Functions

```rust
/// Search GitHub for alacritty theme repositories
pub async fn search_themes(client: &Client, query: &str) -> Result<Vec<SearchItem>>;

/// Fetch releases/tags from a specific repository
pub async fn get_repo_releases(client: &Client, repo: &str) -> Result<Vec<ThemeRelease>>;

/// Get download URL for theme archive from a repo
pub async fn get_theme_archive_url(client: &Client, repo: &str, branch: Option<&str>) -> Result<String>;
```

---

## 3. New Module: `src/downloader.rs`

### Purpose
Handle downloading theme archives with progress bar display.

### Key Structures

```rust
use indicatif::{ProgressBar, ProgressStyle};

/// Wrapper for progress bar with download metadata
struct DownloadProgress {
    pb: ProgressBar,
    total_size: u64,
    downloaded: u64,
}

impl DownloadProgress {
    /// Create new progress bar with style template
    fn new(total_size: u64, filename: &str) -> Self;
    
    /// Update progress with new byte count
    fn update(&self, bytes: u64);
    
    /// Finish and clear the progress bar
    fn finish(&self);
}
```

### Key Functions

```rust
/// Download a theme archive with progress display
pub async fn download_with_progress(
    client: &Client,
    url: &str,
    destination: &Path,
    filename: &str,
) -> Result<PathBuf>;

/// Extract downloaded theme archive
pub async fn extract_theme_archive(archive: &Path, themes_dir: &Path) -> Result<Vec<PathBuf>>;
```

### Progress Bar Style (from bob)
```rust
ProgressStyle::with_template(
    "{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
)
.progress_chars("█  ")
```

---

## 4. New Module: `src/remote.rs`

### Purpose
High-level interface for remote theme operations - combines github and downloader.

### Key Functions

```rust
use crate::github::{SearchItem, ThemeRelease};

/// List available themes from GitHub search
pub async fn list_remote_themes(query: Option<&str>) -> Result<Vec<SearchItem>>;

/// Download themes from a repository
pub async fn download_repository_themes(
    repo: &str,
    themes_dir: &Path,
    branch: Option<&str>,
) -> Result<Vec<String>>;

/// Clone/download from a specific GitHub URL
pub async fn download_from_url(
    url: &str,
    themes_dir: &Path,
) -> Result<Vec<String>>;
```

---

## 5. CLI Extension: `src/config/cli.rs`

### Add New Commands

```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands ...
    
    /// Search for themes on GitHub
    Search {
        /// Search query
        #[arg()]
        query: Option<String>,
    },
    
    /// Download themes from a GitHub repository
    Download {
        /// Repository in format "owner/repo" or full GitHub URL
        #[arg()]
        repository: String,
        
        /// Branch to download from [default: main]
        #[arg(long, short)]
        branch: Option<String>,
        
        /// Overwrite existing themes
        #[arg(long, short)]
        force: bool,
    },
}
```

---

## 6. Error Handling: `src/error.rs`

### Add New Error Variants

```rust
pub enum AlthemerError {
    // ... existing variants ...
    
    GitHubApiError(String),
    DownloadError(String),
    ExtractionError(String),
    RateLimitExceeded,
}
```

---

## 7. Theme Directory Structure

### Expected Layout After Download
```
themes_dir/
├── dracula.toml          # Existing local themes
├── nord.toml
├── github-user-repo/     # Directory from GitHub download
│   ├── tokyo-night.toml
│   ├── tokyo-night-storm.toml
│   └── tokyo-night-day.toml
└── another-repo/
    ├── catppuccin-mocha.toml
    └── catppuccin-latte.toml
```

---

## 8. Implementation Phases

### Phase 1: Foundation
1. Add dependencies to `Cargo.toml`
2. Create `src/github.rs` module
3. Create `src/error.rs` variants
4. Create `src/downloader.rs` module
5. Test basic GitHub API calls

### Phase 2: Core Download
1. Implement `download_with_progress` function
2. Implement archive extraction
3. Add basic download command to CLI
4. Test end-to-end download flow

### Phase 3: Search & Discovery
1. Implement GitHub search functionality
2. Add `search` command to CLI
3. Integrate with TUI picker for search results
4. Add "download" action to picker

### Phase 4: Polish
1. Add rate limiting handling (exponential backoff)
2. Add progress for multi-file downloads
3. Add GitHub token support for higher rate limits
4. Documentation and error messages

---

## 9. File Structure Summary

```
althemer/
├── src/
│   ├── main.rs              # Update to include new modules
│   ├── config/
│   │   ├── mod.rs           # Export new CLI commands
│   │   └── cli.rs           # Add Search, Download commands
│   ├── github.rs            # NEW: GitHub API interactions
│   ├── downloader.rs        # NEW: Progress bar downloads
│   ├── remote.rs            # NEW: High-level remote operations
│   ├── themes.rs            # Update: Add import from remote
│   ├── switcher.rs          # Minor: Handle remote themes
│   ├── error.rs             # Update: Add new error types
│   └── tui.rs               # Minor: Add remote theme support
├── Cargo.toml               # Update: Add dependencies
└── IMPLEMENTATION_PLAN.md   # This file
```

---

## 10. Key Implementation Details from bob

### GitHub Request Pattern
```rust
async fn make_github_request<T: AsRef<str> + reqwest::IntoUrl>(
    client: &Client,
    url: T,
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
```

### Streaming Download with Progress
```rust
let response = client.get(url).send().await?;
let total_size = response.content_length().unwrap_or(0);
let mut stream = response.bytes_stream();
let pb = ProgressBar::new(total_size);

let mut file = File::create(dest).await?;
while let Some(chunk) = stream.next().await {
    let bytes = chunk?;
    file.write_all(&bytes).await?;
    pb.set_position(file.metadata().await?.len());
}
```

### Rate Limit Handling
```rust
if value.get("message").is_some() {
    let error: ErrorResponse = serde_json::from_value(value)?;
    if result.documentation_url.contains("rate-limiting") {
        return Err(anyhow!("GitHub API rate limit reached"));
    }
}
```

---

## 11. Popular Alacritty Theme Repos (for testing)

- `alacritty-theme/alacritty-theme` - Official theme collection
- `eendroroy/alacritty` - Many themes
- `josediegogallardo/awesome-alacritty` - Aggregated themes
- `catppuccin/alacritty` - Catppuccin themes
- `zatchheems/alacritty-themes-collection` - Many themes

---

## 12. API Endpoints to Use

### Search Repositories
```
GET https://api.github.com/search/repositories?q=alacritty+theme+in:readme&sort=stars
```

### Get Repository Info
```
GET https://api.github.com/repos/{owner}/{repo}
```

### Get Repository Contents (for raw .toml files)
```
GET https://api.github.com/repos/{owner}/{repo}/contents?ref={branch}
```

### Get Archive URL
```
GET https://api.github.com/repos/{owner}/{repo}/zipball/{branch}
```

---

## 13. Questions / Decisions Needed

1. **GitHub Token**: Support optional token for higher rate limits?
2. **Source Format**: Support downloading individual .toml files or only archives?
3. **Conflict Handling**: How to handle theme name collisions?
4. **Categories**: Should downloaded themes be auto-categorized?
5. **Cleanup**: Delete downloaded archives after extraction?

---

## 14. Testing Strategy

1. Unit tests for GitHub API parsing
2. Integration tests with mock responses
3. Manual testing with real GitHub API
4. End-to-end test with actual theme download

---

## Next Steps

1. Review and approve this plan
2. Start Phase 1 implementation
3. Begin with basic GitHub API module
4. Add download functionality incrementally
