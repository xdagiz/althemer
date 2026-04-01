use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

use crate::error::{AlthemerError, Result};

const DEFAULT_REPO: &str = "alacritty/alacritty-theme";
const DEFAULT_BRANCH: &str = "master";

#[derive(Debug, Deserialize)]
struct TreeItem {
    path: String,
}

#[derive(Debug, Deserialize)]
struct TreeResponse {
    tree: Vec<TreeItem>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug, Clone)]
struct RepoInfo {
    owner: String,
    repo: String,
}

fn parse_github_url(url: &str) -> Option<RepoInfo> {
    if let Some(caps) =
        regex_lite::Regex::new(r"^https?://github\.com/([^/]+)/([^/]+?)(?:\.git)?/?$")
            .ok()?
            .captures(url)
    {
        return Some(RepoInfo {
            owner: caps.get(1)?.as_str().to_string(),
            repo: caps.get(2)?.as_str().to_string(),
        });
    }

    if let Some(caps) = regex_lite::Regex::new(r"^git@github\.com:([^/]+)/(.+?)(?:\.git)?$")
        .ok()?
        .captures(url)
    {
        return Some(RepoInfo {
            owner: caps.get(1)?.as_str().to_string(),
            repo: caps.get(2)?.as_str().to_string(),
        });
    }

    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() == 2 && !url.contains("github.com") {
        return Some(RepoInfo {
            owner: parts[0].to_string(),
            repo: parts[1].to_string(),
        });
    }

    None
}

fn resolve_repo(repo: Option<&str>) -> Result<(String, String)> {
    match repo {
        Some(url) => {
            if let Some(info) = parse_github_url(url) {
                Ok((info.owner, info.repo))
            } else {
                let parts: Vec<&str> = url.split('/').collect();
                if parts.len() == 2 {
                    Ok((parts[0].to_string(), parts[1].to_string()))
                } else {
                    Err(AlthemerError::InvalidRepoUrl(url.to_string()))
                }
            }
        }
        None => {
            let info = parse_github_url(DEFAULT_REPO)
                .ok_or_else(|| AlthemerError::InvalidRepoUrl(DEFAULT_REPO.to_string()))?;
            Ok((info.owner, info.repo))
        }
    }
}

pub async fn make_github_request<T: AsRef<str> + reqwest::IntoUrl>(
    client: &Client,
    url: T,
) -> Result<String> {
    client
        .get(url)
        .header("user-agent", "althemer")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| AlthemerError::GitHubApi(e.to_string()))?
        .text()
        .await
        .map_err(|e| AlthemerError::GitHubApi(e.to_string()))
}

pub fn deserialize_response<T: DeserializeOwned>(response: &str) -> Result<T> {
    let value: serde_json::Value = serde_json::from_str(response)
        .map_err(|e| AlthemerError::GitHubApi(format!("Failed to parse JSON: {}", e)))?;

    if value.get("message").is_some() {
        let result: ErrorResponse = serde_json::from_value(value)?;
        return Err(AlthemerError::GitHubApi(result.message));
    }

    serde_json::from_value(value)
        .map_err(|e| AlthemerError::GitHubApi(format!("Failed to deserialize: {}", e)))
}

async fn list_repo_tree(
    client: &Client,
    owner: &str,
    repo: &str,
    branch: &str,
) -> Result<Vec<TreeItem>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
        owner, repo, branch
    );

    let response = make_github_request(client, &url).await?;
    let tree_response: TreeResponse = deserialize_response(&response)?;

    Ok(tree_response.tree)
}

fn filter_toml_files(tree: &[TreeItem]) -> Vec<&TreeItem> {
    tree.iter()
        .filter(|item| item.path.ends_with(".toml"))
        .collect()
}

#[derive(Debug, Clone)]
struct DownloadFile {
    path: String,
    url: String,
}

fn get_filename(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("theme.toml")
}

async fn download_file(client: &Client, url: &str, dest: &Path) -> Result<PathBuf> {
    let response = client
        .get(url)
        .header("user-agent", "althemer")
        .send()
        .await
        .map_err(|e| AlthemerError::Download(e.to_string()))?;

    let mut dest_file = tokio::fs::File::create(dest)
        .await
        .map_err(|e| AlthemerError::Download(e.to_string()))?;
    let mut response_bytes = response.bytes_stream();

    while let Some(chunk) = response_bytes.next().await {
        let bytes = chunk.map_err(|e| AlthemerError::Download(e.to_string()))?;
        dest_file.write_all(&bytes).await?;
    }

    Ok(dest.to_path_buf())
}

#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub repo: Option<String>,
    pub branch: Option<String>,
    pub force: bool,
}

pub async fn download_themes(
    client: &Client,
    themes_dir: &Path,
    options: &DownloadOptions,
) -> Result<Vec<String>> {
    let (owner, repo) = resolve_repo(options.repo.as_deref())?;
    let branch = options.branch.as_deref().unwrap_or(DEFAULT_BRANCH);
    let tree = list_repo_tree(client, &owner, &repo, branch).await?;
    let toml_files = filter_toml_files(&tree);

    if toml_files.is_empty() {
        return Err(AlthemerError::Download(format!(
            "No TOML files found in {}/{} on branch '{}'",
            owner, repo, branch
        )));
    }

    tokio::fs::create_dir_all(themes_dir)
        .await
        .map_err(|e| AlthemerError::Download(format!("Failed to create directory: {}", e)))?;

    let downloads = toml_files
        .iter()
        .map(|&item| {
            let path = item.path.to_string();
            let url = format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}",
                owner, repo, branch, path
            );

            DownloadFile { path, url }
        })
        .collect::<Vec<_>>();

    let total_files = downloads.len();
    let mut downloaded_paths = Vec::with_capacity(total_files);
    let pb = ProgressBar::new(total_files as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} Downloading [{wide_bar:.green}] {msg}")
            .map_err(|_| AlthemerError::Download("Failed to set progress bar style".to_string()))
            .unwrap()
            .progress_chars("█ "),
    );

    for file in downloads.iter() {
        let filename = get_filename(&file.path);
        let dest = themes_dir.join(filename);

        pb.set_message(filename.to_string());

        if dest.exists() && !options.force {
            pb.inc(1);
            continue;
        }

        match download_file(client, &file.url, &dest).await {
            Ok(path) => {
                pb.inc(1);
                downloaded_paths.push(path)
            }
            Err(e) => eprintln!("Warning: Failed to download {}: {}", file.path, e),
        }
    }

    if downloaded_paths.is_empty() {
        return Err(AlthemerError::Download(
            "No themes were downloaded (try using --force to override)".to_string(),
        ));
    }

    pb.finish_and_clear();

    let theme_names = downloaded_paths
        .iter()
        .filter_map(|p| {
            p.file_stem()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .collect::<Vec<_>>();

    Ok(theme_names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_https_url() {
        let url = "https://github.com/alacritty/alacritty-theme";
        let info = parse_github_url(url).unwrap();
        assert_eq!(info.owner, "alacritty");
        assert_eq!(info.repo, "alacritty-theme");
    }

    #[test]
    fn test_parse_ssh_url() {
        let url = "git@github.com:user/my-repo.git";
        let info = parse_github_url(url).unwrap();
        assert_eq!(info.owner, "user");
        assert_eq!(info.repo, "my-repo");
    }

    #[test]
    fn test_parse_shorthand() {
        let url = "owner/repo";
        let info = parse_github_url(url).unwrap();
        assert_eq!(info.owner, "owner");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn test_resolve_repo_with_url() {
        let result = resolve_repo(Some("https://github.com/catppuccin/alacritty")).unwrap();
        assert_eq!(result.0, "catppuccin");
        assert_eq!(result.1, "alacritty");
    }

    #[test]
    fn test_resolve_repo_default() {
        let result = resolve_repo(None).unwrap();
        assert_eq!(result.0, "alacritty");
        assert_eq!(result.1, "alacritty-theme");
    }
}
