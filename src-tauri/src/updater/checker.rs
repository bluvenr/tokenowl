use serde::{Deserialize, Serialize};
use crate::error::AppResult;

/// CDN-hosted latest version info (remote/latest.json on GitHub, served via jsDelivr)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestRelease {
    pub version: String,
    pub release_url: Option<String>,
    pub changelog: Option<String>,
}

/// Version check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_url: Option<String>,
    pub changelog: Option<String>,
}

/// Update checker - fetches latest version from jsDelivr CDN (backed by GitHub repo)
pub struct UpdateChecker {
    cdn_url: String,
    current_version: String,
}

impl UpdateChecker {
    pub fn new(owner: &str, repo: &str, current_version: &str) -> Self {
        // Use jsDelivr CDN for fast global access (especially in China)
        let cdn_url = format!(
            "https://cdn.jsdelivr.net/gh/{}/{}@main/remote/latest.json",
            owner, repo
        );
        Self {
            cdn_url,
            current_version: current_version.to_string(),
        }
    }

    /// Check for updates via CDN
    pub async fn check(&self) -> AppResult<UpdateInfo> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        match client
            .get(&self.cdn_url)
            .header("Cache-Control", "no-cache")
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    let release: LatestRelease = response
                        .json()
                        .await
                        .unwrap_or(LatestRelease {
                            version: "0.0.0".to_string(),
                            release_url: None,
                            changelog: None,
                        });

                    let latest_version = release.version.trim_start_matches('v').to_string();
                    let update_available = is_newer_version(&latest_version, &self.current_version);

                    Ok(UpdateInfo {
                        current_version: self.current_version.clone(),
                        latest_version,
                        update_available,
                        release_url: if update_available { release.release_url } else { None },
                        changelog: if update_available { release.changelog } else { None },
                    })
                } else {
                    log::warn!(
                        "CDN update check returned status {}",
                        response.status()
                    );
                    Ok(self.no_update_result())
                }
            }
            Err(e) => {
                log::warn!("Failed to check for updates via CDN: {}", e);
                Ok(self.no_update_result())
            }
        }
    }

    fn no_update_result(&self) -> UpdateInfo {
        UpdateInfo {
            current_version: self.current_version.clone(),
            latest_version: self.current_version.clone(),
            update_available: false,
            release_url: None,
            changelog: None,
        }
    }
}

/// Parse a semver string into (major, minor, patch, pre_release)
/// Pre-release suffix (e.g. "-beta.1") is preserved as Option<String>
/// Pre-release versions are considered *older* than the same version without pre-release
fn parse_semver(v: &str) -> (u32, u32, u32, Option<&str>) {
    let (version_part, pre_release) = match v.find('-') {
        Some(idx) => (&v[..idx], Some(&v[idx + 1..])),
        None => (v, None),
    };

    let parts: Vec<&str> = version_part.split('.').collect();
    (
        parts.first().and_then(|p| p.parse().ok()).unwrap_or(0),
        parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(0),
        parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0),
        pre_release,
    )
}

/// Compare two semver strings. Returns true if `latest` is newer than `current`.
/// Pre-release versions are considered older than the same version without pre-release.
fn is_newer_version(latest: &str, current: &str) -> bool {
    let l = parse_semver(latest);
    let c = parse_semver(current);

    let l_nums = (l.0, l.1, l.2);
    let c_nums = (c.0, c.1, c.2);

    // Different version numbers — simple numeric comparison
    if l_nums != c_nums {
        return l_nums > c_nums;
    }

    // Same major.minor.patch — pre-release is OLDER than stable
    // e.g. 1.1.0-beta.1 < 1.1.0
    match (l.3, c.3) {
        (None, Some(_)) => true,   // latest is stable, current is pre-release → latest is newer
        (Some(_), None) => false,  // latest is pre-release, current is stable → latest is NOT newer
        (Some(lp), Some(cp)) => lp > cp, // both pre-release → lexicographic compare
        (None, None) => false,     // same stable version → not newer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newer_version() {
        assert!(is_newer_version("1.1.0", "1.0.0"));
        assert!(is_newer_version("2.0.0", "1.9.9"));
        assert!(is_newer_version("1.0.1", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
        assert!(!is_newer_version("0.9.0", "1.0.0"));
    }

    #[test]
    fn test_prerelease() {
        assert!(!is_newer_version("1.0.0-beta.1", "1.0.0"));
        assert!(is_newer_version("1.0.0", "1.0.0-beta.1"));
        assert!(is_newer_version("1.0.0-beta.2", "1.0.0-beta.1"));
        assert!(is_newer_version("1.1.0", "1.0.0-rc.1"));
    }

    #[test]
    fn test_v_prefix_stripped() {
        assert!(is_newer_version("1.1.0", "1.0.0"));
    }
}
