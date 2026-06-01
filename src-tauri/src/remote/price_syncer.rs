use crate::models::settings::ModelPricing;
use crate::remote::download_source::{DownloadSource, SharedDownloadSource};
use std::time::{Duration, Instant};
use std::sync::Mutex;

/// Cached remote prices
struct CachedPrices {
    prices: Vec<ModelPricing>,
    fetched_at: Instant,
}

pub struct PriceSyncer {
    cache: Mutex<Option<CachedPrices>>,
    cache_duration: Duration,
    github_owner: String,
    github_repo: String,
    download_source: SharedDownloadSource,
    client: reqwest::Client,
}

impl PriceSyncer {
    pub fn new(owner: &str, repo: &str, interval_hours: u8, download_source: SharedDownloadSource) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self {
            cache: Mutex::new(None),
            cache_duration: Duration::from_secs(interval_hours as u64 * 3600),
            github_owner: owner.to_string(),
            github_repo: repo.to_string(),
            download_source,
            client,
        }
    }

    /// Fetch remote prices (with caching)
    pub async fn sync_prices(&self) -> Vec<ModelPricing> {
        // Check cache first
        {
            let cache = match self.cache.lock() {
                Ok(g) => g,
                Err(e) => {
                    log::error!("PriceSyncer cache mutex poisoned: {}", e);
                    return vec![];
                }
            };
            if let Some(cached) = cache.as_ref() {
                if cached.fetched_at.elapsed() < self.cache_duration {
                    log::info!("Using cached remote prices ({} models)", cached.prices.len());
                    return cached.prices.clone();
                }
            }
        }

        // Build URL list based on download source preference
        let source = self.download_source.read()
            .map(|g| g.clone())
            .unwrap_or(DownloadSource::Auto);
        let urls = source.urls_for(&self.github_owner, &self.github_repo, "remote/prices.json");

        // Try each URL in order
        for url in &urls {
            if let Some(p) = self.fetch_prices(url).await {
                self.update_cache(p.clone());
                return p;
            }
            log::warn!("Price fetch failed for: {}, trying next source", url);
        }

        log::warn!("All remote price fetch attempts failed, returning empty");
        vec![]
    }

    async fn fetch_prices(&self, url: &str) -> Option<Vec<ModelPricing>> {
        log::info!("Fetching remote prices from: {}", url);
        match self.client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<Vec<ModelPricing>>().await {
                    Ok(mut prices) => {
                        for p in &mut prices {
                            p.price_source = "remote".to_string();
                        }
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
                log::warn!("Price fetch returned status {}", resp.status());
                None
            }
            Err(e) => {
                log::warn!("Price fetch error: {}", e);
                None
            }
        }
    }

    fn update_cache(&self, prices: Vec<ModelPricing>) {
        // Save to local file cache for next startup
        crate::pricing::registry::save_cached_prices(&prices);

        match self.cache.lock() {
            Ok(mut cache) => {
                *cache = Some(CachedPrices {
                    prices,
                    fetched_at: Instant::now(),
                });
            }
            Err(e) => log::error!("PriceSyncer update_cache mutex poisoned: {}", e),
        }
    }

    /// Get cached prices without fetching
    pub fn get_cached(&self) -> Vec<ModelPricing> {
        match self.cache.lock() {
            Ok(guard) => guard.as_ref().map(|c| c.prices.clone()).unwrap_or_default(),
            Err(e) => {
                log::error!("PriceSyncer get_cached mutex poisoned: {}", e);
                vec![]
            }
        }
    }

    /// Force refresh (bypass cache)
    pub async fn force_sync(&self) -> Vec<ModelPricing> {
        match self.cache.lock() {
            Ok(mut cache) => { *cache = None; }
            Err(e) => log::error!("PriceSyncer force_sync mutex poisoned: {}", e),
        }
        self.sync_prices().await
    }
}
