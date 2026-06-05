use crate::models::settings::ModelPricing;
use crate::remote::download_source::{DownloadSource, SharedDownloadSource};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Remote price syncer — fetches model prices from GitHub/CDN
pub struct PriceSyncer {
    github_owner: String,
    github_repo: String,
    download_source: SharedDownloadSource,
    cache: Mutex<PriceCache>,
    client: reqwest::Client,
}

struct PriceCache {
    prices: Vec<ModelPricing>,
    fetched_at: Option<Instant>,
}

impl PriceSyncer {
    pub fn new(owner: &str, repo: &str, download_source: SharedDownloadSource) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self {
            github_owner: owner.to_string(),
            github_repo: repo.to_string(),
            download_source,
            cache: Mutex::new(PriceCache {
                prices: vec![],
                fetched_at: None,
            }),
            client,
        }
    }

    /// Get cached prices (without fetching from remote)
    pub fn get_cached(&self) -> Vec<ModelPricing> {
        self.cache.lock()
            .map(|c| c.prices.clone())
            .unwrap_or_default()
    }

    /// Force sync: fetch prices from remote and update cache
    pub async fn force_sync(&self) -> Vec<ModelPricing> {
        match self.fetch_prices().await {
            Some(prices) => {
                // Save to local cache file
                crate::pricing::registry::save_cached_prices(&prices);

                // Update in-memory cache
                if let Ok(mut cache) = self.cache.lock() {
                    cache.prices = prices.clone();
                    cache.fetched_at = Some(Instant::now());
                }

                log::info!("Remote price sync: {} models", prices.len());
                prices
            }
            None => {
                log::warn!("Remote price sync failed, returning cached");
                self.get_cached()
            }
        }
    }

    /// Fetch prices from remote JSON
    async fn fetch_prices(&self) -> Option<Vec<ModelPricing>> {
        let source = self.download_source.read()
            .map(|g| g.clone())
            .unwrap_or(DownloadSource::Auto);
        let urls = source.urls_for(&self.github_owner, &self.github_repo, "remote/prices.json");

        for url in &urls {
            if let Some(prices) = self.fetch_url(url).await {
                return Some(prices);
            }
            log::warn!("Price fetch failed for: {}, trying next source", url);
        }

        None
    }

    async fn fetch_url(&self, url: &str) -> Option<Vec<ModelPricing>> {
        log::info!("Fetching remote prices from: {}", url);
        match self.client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<Vec<ModelPricing>>().await {
                    Ok(prices) => {
                        log::info!("Remote prices fetched: {} models", prices.len());
                        Some(prices)
                    }
                    Err(e) => {
                        log::error!("Failed to parse remote prices: {}", e);
                        None
                    }
                }
            }
            Ok(resp) => {
                log::warn!("Remote prices returned status {}", resp.status());
                None
            }
            Err(e) => {
                log::warn!("Remote prices fetch error: {}", e);
                None
            }
        }
    }
}
