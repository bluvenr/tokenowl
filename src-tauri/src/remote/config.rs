use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Remote application configuration (fetched from CDN)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfig {
    /// Minimum supported version (force update below this)
    #[serde(default)]
    pub min_version: String,

    /// Feature flags
    #[serde(default)]
    pub features: FeatureFlags,

    /// Active announcement (null = no announcement)
    #[serde(default)]
    pub announcement: Option<Announcement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FeatureFlags {
    /// Enable remote price sync
    #[serde(default = "default_true")]
    pub price_sync: bool,

    /// Enable update checker
    #[serde(default = "default_true")]
    pub update_check: bool,

    /// Enable crash reporting
    #[serde(default = "default_true")]
    pub crash_report: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Announcement {
    pub id: String,
    pub title: String,
    pub message: String,
    #[serde(default)]
    pub link: Option<String>,
    /// ISO date string
    #[serde(default)]
    pub dismissible: bool,
}

/// Cached remote config with expiry
struct CachedConfig {
    config: RemoteConfig,
    fetched_at: Instant,
}

pub struct RemoteConfigManager {
    cache: Mutex<Option<CachedConfig>>,
    cache_duration: Duration,
    github_owner: String,
    github_repo: String,
    client: reqwest::Client,
}

impl RemoteConfigManager {
    pub fn new(owner: &str, repo: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self {
            cache: Mutex::new(None),
            cache_duration: Duration::from_secs(6 * 3600), // 6 hours
            github_owner: owner.to_string(),
            github_repo: repo.to_string(),
            client,
        }
    }

    /// Get CDN URL for a file, with optional fallback
    fn cdn_url(&self, file: &str) -> String {
        format!(
            "https://cdn.jsdelivr.net/gh/{}/{}/remote/{}",
            self.github_owner, self.github_repo, file
        )
    }

    fn fallback_url(&self, file: &str) -> String {
        format!(
            "https://raw.githubusercontent.com/{}/{}/main/remote/{}",
            self.github_owner, self.github_repo, file
        )
    }

    /// Fetch remote config.json (with caching)
    pub async fn fetch_config(&self) -> Option<RemoteConfig> {
        // Check cache first
        {
            let cache = self.cache.lock().ok()?;
            if let Some(cached) = cache.as_ref() {
                if cached.fetched_at.elapsed() < self.cache_duration {
                    return Some(cached.config.clone());
                }
            }
        }

        // Fetch from CDN (with fallback)
        let config = self.fetch_from_cdn("config.json").await;

        if let Some(cfg) = config {
            if let Ok(mut cache) = self.cache.lock() {
                *cache = Some(CachedConfig {
                    config: cfg.clone(),
                    fetched_at: Instant::now(),
                });
            }
            Some(cfg)
        } else {
            // Try fallback URL
            log::warn!("CDN config fetch failed, trying fallback");
            match self.fetch_fallback("config.json").await {
                Some(cfg) => {
                    if let Ok(mut cache) = self.cache.lock() {
                        *cache = Some(CachedConfig {
                            config: cfg.clone(),
                            fetched_at: Instant::now(),
                        });
                    }
                    Some(cfg)
                }
                None => {
                    log::warn!("All remote config fetch attempts failed");
                    None
                }
            }
        }
    }

    async fn fetch_from_cdn(&self, file: &str) -> Option<RemoteConfig> {
        let url = self.cdn_url(file);
        log::info!("Fetching remote config from CDN: {}", url);
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<RemoteConfig>().await {
                    Ok(config) => {
                        log::info!("Remote config fetched successfully");
                        Some(config)
                    }
                    Err(e) => {
                        log::error!("Failed to parse remote config: {}", e);
                        None
                    }
                }
            }
            Ok(resp) => {
                log::warn!("CDN returned status {}", resp.status());
                None
            }
            Err(e) => {
                log::warn!("CDN fetch error: {}", e);
                None
            }
        }
    }

    async fn fetch_fallback(&self, file: &str) -> Option<RemoteConfig> {
        let url = self.fallback_url(file);
        log::info!("Fetching remote config from fallback: {}", url);
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                resp.json::<RemoteConfig>().await.ok()
            }
            _ => None,
        }
    }

    /// Get the currently cached config (without fetching)
    pub fn get_cached(&self) -> Option<RemoteConfig> {
        self.cache.lock().ok()?.as_ref().map(|c| c.config.clone())
    }

    /// Get the local cache file path
    pub fn cache_path() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("tokenowl").join("remote_config_cache.json"))
    }
}
