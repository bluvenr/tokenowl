use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Version info from remote (latest.json in GitHub Releases)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteVersion {
    pub version: String,
    pub notes: Option<String>,
    pub pub_date: Option<String>,
    /// Platform-specific download URLs
    #[serde(default)]
    pub platforms: PlatformUrls,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlatformUrls {
    #[serde(rename = "windows-x86_64")]
    pub windows_x86_64: Option<PlatformDownload>,
    #[serde(rename = "darwin-x86_64")]
    pub darwin_x86_64: Option<PlatformDownload>,
    #[serde(rename = "darwin-aarch64")]
    pub darwin_aarch64: Option<PlatformDownload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformDownload {
    pub url: String,
    pub signature: Option<String>,
}

/// Update info to send to frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current_version: String,
    pub new_version: String,
    pub notes: String,
    pub download_url: String,
}

pub struct UpdateChecker {
    current_version: String,
    github_owner: String,
    github_repo: String,
    client: reqwest::Client,
}

impl UpdateChecker {
    pub fn new(current_version: &str, owner: &str, repo: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self {
            current_version: current_version.to_string(),
            github_owner: owner.to_string(),
            github_repo: repo.to_string(),
            client,
        }
    }

    fn latest_json_url(&self) -> String {
        format!(
            "https://cdn.jsdelivr.net/gh/{}/{}/remote/latest.json",
            self.github_owner, self.github_repo
        )
    }

    fn fallback_latest_url(&self) -> String {
        format!(
            "https://raw.githubusercontent.com/{}/{}/main/remote/latest.json",
            self.github_owner, self.github_repo
        )
    }

    /// Check for updates. Returns Some(UpdateInfo) if a newer version is available.
    pub async fn check_for_update(&self) -> Option<UpdateInfo> {
        let remote = self.fetch_latest().await?;

        if self.is_newer(&remote.version) {
            let download_url = self.get_download_url(&remote);
            Some(UpdateInfo {
                current_version: self.current_version.clone(),
                new_version: remote.version,
                notes: remote.notes.unwrap_or_default(),
                download_url,
            })
        } else {
            log::info!("Application is up to date (v{})", self.current_version);
            None
        }
    }

    async fn fetch_latest(&self) -> Option<RemoteVersion> {
        // Try CDN first
        if let Some(v) = self.fetch_url(&self.latest_json_url()).await {
            return Some(v);
        }
        // Fallback
        self.fetch_url(&self.fallback_latest_url()).await
    }

    async fn fetch_url(&self, url: &str) -> Option<RemoteVersion> {
        log::info!("Checking for updates at: {}", url);

        match self.client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<RemoteVersion>().await {
                    Ok(version) => Some(version),
                    Err(e) => {
                        log::error!("Failed to parse latest.json: {}", e);
                        None
                    }
                }
            }
            Ok(resp) => {
                log::warn!("Update check returned status {}", resp.status());
                None
            }
            Err(e) => {
                log::warn!("Update check failed: {}", e);
                None
            }
        }
    }

    /// Compare semver: returns true if `remote` > `current`
    fn is_newer(&self, remote: &str) -> bool {
        let current = parse_semver(&self.current_version);
        let remote_v = parse_semver(remote);
        remote_v > current
    }

    fn get_download_url(&self, remote: &RemoteVersion) -> String {
        // Try platform-specific URL first
        #[cfg(target_os = "windows")]
        if let Some(ref win) = remote.platforms.windows_x86_64 {
            return win.url.clone();
        }

        #[cfg(target_os = "macos")]
        {
            #[cfg(target_arch = "aarch64")]
            if let Some(ref mac) = remote.platforms.darwin_aarch64 {
                return mac.url.clone();
            }
            #[cfg(target_arch = "x86_64")]
            if let Some(ref mac) = remote.platforms.darwin_x86_64 {
                return mac.url.clone();
            }
        }

        // Fallback to GitHub releases page
        format!(
            "https://github.com/{}/{}/releases/latest",
            self.github_owner, self.github_repo
        )
    }

    /// Start periodic update checking (spawned as background task)
    pub fn start_periodic_check(
        owner: String,
        repo: String,
        current_version: String,
        interval_hours: u8,
        app_handle: tauri::AppHandle,
    ) {
        if interval_hours == 0 {
            log::info!("Update check disabled (interval = 0)");
            return;
        }

        tauri::async_runtime::spawn(async move {
            let checker = UpdateChecker::new(&current_version, &owner, &repo);
            let interval = Duration::from_secs(interval_hours as u64 * 3600);

            // Initial check after 5 second delay
            tokio::time::sleep(Duration::from_secs(5)).await;

            loop {
                if let Some(update) = checker.check_for_update().await {
                    log::info!("Update available: v{} -> v{}", update.current_version, update.new_version);
                    use tauri::Emitter;
                    let _ = app_handle.emit("tokenowl:update-available", &update);
                }

                tokio::time::sleep(interval).await;
            }
        });
    }
}

/// Parse a semver string into (major, minor, patch) tuple
fn parse_semver(version: &str) -> (u32, u32, u32) {
    let v = version.trim_start_matches('v');
    let parts: Vec<&str> = v.split('.').collect();
    let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_comparison() {
        assert!(parse_semver("1.1.0") > parse_semver("1.0.0"));
        assert!(parse_semver("2.0.0") > parse_semver("1.9.9"));
        assert!(parse_semver("1.0.1") > parse_semver("1.0.0"));
        assert!(!(parse_semver("1.0.0") > parse_semver("1.0.0")));
    }

    #[test]
    fn test_parse_semver_with_v_prefix() {
        assert_eq!(parse_semver("v1.2.3"), (1, 2, 3));
        assert_eq!(parse_semver("0.1.0"), (0, 1, 0));
    }
}
