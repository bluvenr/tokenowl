use std::sync::{Arc, RwLock};

/// Gitee repository owner (mirror of GitHub repo)
pub const GITEE_OWNER: &str = "bluvenr";
/// Gitee repository name
pub const GITEE_REPO: &str = "tokenowl";

/// Download source preference for remote resource fetching
#[derive(Debug, Clone, PartialEq)]
pub enum DownloadSource {
    /// Auto: use CDN first, then GitHub Raw fallback (default)
    Auto,
    /// GitHub: same as Auto, CDN first then GitHub Raw
    GitHub,
    /// Gitee: use Gitee Raw first, then CDN fallback (for domestic China users)
    Gitee,
}

impl DownloadSource {
    /// Parse from a string value (stored in DB as "auto", "github", "gitee")
    pub fn from_str(s: &str) -> Self {
        match s {
            "github" => Self::GitHub,
            "gitee" => Self::Gitee,
            _ => Self::Auto,
        }
    }

    /// Convert back to string for DB storage
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::GitHub => "github",
            Self::Gitee => "gitee",
        }
    }

    /// Build jsDelivr CDN URL for a file path (mirrors GitHub repo)
    pub fn cdn_url(github_owner: &str, github_repo: &str, path: &str) -> String {
        format!(
            "https://cdn.jsdelivr.net/gh/{}/{}/{}",
            github_owner, github_repo, path
        )
    }

    /// Build GitHub Raw URL for a file path
    pub fn github_raw_url(github_owner: &str, github_repo: &str, path: &str) -> String {
        format!(
            "https://raw.githubusercontent.com/{}/{}/main/{}",
            github_owner, github_repo, path
        )
    }

    /// Build Gitee Raw URL for a file path (mirror repo)
    pub fn gitee_raw_url(path: &str) -> String {
        format!(
            "https://gitee.com/{}/{}/raw/main/{}",
            GITEE_OWNER, GITEE_REPO, path
        )
    }

    /// Get ordered list of URLs to try based on download source preference
    pub fn urls_for(&self, github_owner: &str, github_repo: &str, path: &str) -> Vec<String> {
        match self {
            Self::Auto | Self::GitHub => vec![
                Self::cdn_url(github_owner, github_repo, path),
                Self::github_raw_url(github_owner, github_repo, path),
            ],
            Self::Gitee => vec![
                Self::gitee_raw_url(path),
                Self::cdn_url(github_owner, github_repo, path),
            ],
        }
    }
}

/// Thread-safe shared download source that can be updated at runtime
pub type SharedDownloadSource = Arc<RwLock<DownloadSource>>;

/// Create a new shared download source from a string value
pub fn new_shared(source: &str) -> SharedDownloadSource {
    Arc::new(RwLock::new(DownloadSource::from_str(source)))
}

/// Update a shared download source from a string value
pub fn update_shared(shared: &SharedDownloadSource, source: &str) {
    if let Ok(mut guard) = shared.write() {
        *guard = DownloadSource::from_str(source);
        log::info!("Download source updated to: {}", source);
    }
}
