use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::error::AppResult;

/// Remote configuration from GitHub
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteConfig {
    pub min_version: Option<String>,
    pub announcement: Option<String>,
    pub features: HashMap<String, bool>,
    pub messages: HashMap<String, String>,
}

/// Remote config manager - fetches config from GitHub/jsDelivr
pub struct ConfigManager {
    config: Option<RemoteConfig>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Fetch remote config from CDN
    pub async fn fetch(&mut self) -> AppResult<RemoteConfig> {
        let url = "https://cdn.jsdelivr.net/gh/bluvenr/tokenowl@main/remote/config.json";

        match reqwest::get(url).await {
            Ok(response) => {
                if response.status().is_success() {
                    let config: RemoteConfig = response.json().await.unwrap_or_default();
                    self.config = Some(config.clone());
                    Ok(config)
                } else {
                    log::warn!("Failed to fetch remote config: HTTP {}", response.status());
                    Ok(self.config.clone().unwrap_or_default())
                }
            }
            Err(e) => {
                log::warn!("Failed to fetch remote config: {}", e);
                Ok(self.config.clone().unwrap_or_default())
            }
        }
    }

    /// Get cached config
    pub fn get_config(&self) -> Option<&RemoteConfig> {
        self.config.as_ref()
    }

    /// Check if a feature is enabled remotely
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.config
            .as_ref()
            .and_then(|c| c.features.get(feature))
            .copied()
            .unwrap_or(true) // Default to enabled if not specified
    }
}
