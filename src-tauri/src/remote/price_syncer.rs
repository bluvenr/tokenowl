use crate::models::settings::ModelPricing;
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
    client: reqwest::Client,
}

impl PriceSyncer {
    pub fn new(owner: &str, repo: &str, interval_hours: u8) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self {
            cache: Mutex::new(None),
            cache_duration: Duration::from_secs(interval_hours as u64 * 3600),
            github_owner: owner.to_string(),
            github_repo: repo.to_string(),
            client,
        }
    }

    fn cdn_url(&self) -> String {
        format!(
            "https://cdn.jsdelivr.net/gh/{}/{}/remote/prices.json",
            self.github_owner, self.github_repo
        )
    }

    fn fallback_url(&self) -> String {
        format!(
            "https://raw.githubusercontent.com/{}/{}/main/remote/prices.json",
            self.github_owner, self.github_repo
        )
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

        // Try CDN first
        let prices = self.fetch_prices(&self.cdn_url()).await;

        if let Some(p) = prices {
            self.update_cache(p.clone());
            return p;
        }

        // Fallback to GitHub Raw
        log::warn!("CDN price fetch failed, trying fallback");
        if let Some(p) = self.fetch_prices(&self.fallback_url()).await {
            self.update_cache(p.clone());
            return p;
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
