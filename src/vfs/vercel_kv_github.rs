use crate::vfs::VfsError;
use serde::Deserialize;

/// GitHub API tree response structure
#[derive(Debug, Deserialize)]
pub struct GitHubTreeResponse {
    pub tree: Vec<GitHubTreeItem>,
    pub truncated: bool,
}

/// GitHub API tree item structure
#[derive(Debug, Deserialize)]
pub struct GitHubTreeItem {
    pub path: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub size: Option<usize>,
}

// Configuration for GitHub raw content source
#[derive(Clone, Debug)]
pub struct GitHubConfig {
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub root_path: String,
}

impl GitHubConfig {
    pub fn from_env() -> Result<Self, VfsError> {
        Ok(Self {
            owner: std::env::var("VFS_GITHUB_OWNER").unwrap_or_else(|_| "lonelam".to_string()),
            repo: std::env::var("VFS_GITHUB_REPO")
                .unwrap_or_else(|_| "subconverter-rs".to_string()),
            branch: std::env::var("VFS_GITHUB_BRANCH").unwrap_or_else(|_| "main".to_string()),
            root_path: std::env::var("VFS_GITHUB_ROOT_PATH").unwrap_or_else(|_| "base".to_string()),
        })
    }

    pub fn get_raw_url(&self, file_path: &str) -> String {
        let base = format!(
            "https://raw.githubusercontent.com/{}/{}/{}",
            self.owner, self.repo, self.branch
        );
        let full_path = if self.root_path.is_empty() {
            file_path.to_string()
        } else {
            format!("{}/{}", self.root_path.trim_matches('/'), file_path)
        };
        format!("{}/{}", base, full_path.trim_start_matches('/'))
    }
}
