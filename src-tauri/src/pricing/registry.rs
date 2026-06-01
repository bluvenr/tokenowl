use crate::models::settings::ModelPricing;
use std::path::PathBuf;

/// Get the local price cache file path
fn cache_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| {
        d.join(crate::APP_DATA_DIR).join("price_cache.json")
    })
}

/// Load cached prices from local file (written after last successful remote fetch)
pub fn load_cached_prices() -> Vec<ModelPricing> {
    let path = match cache_path() {
        Some(p) if p.exists() => p,
        _ => return vec![],
    };
    let json = match std::fs::read_to_string(&path) {
        Ok(j) => j,
        Err(e) => {
            log::warn!("Failed to read price cache: {}", e);
            return vec![];
        }
    };
    let mut prices: Vec<ModelPricing> = match serde_json::from_str(&json) {
        Ok(p) => p,
        Err(e) => {
            log::warn!("Failed to parse price cache: {}", e);
            return vec![];
        }
    };
    for p in &mut prices {
        p.price_source = "cached".to_string();
    }
    log::info!("Loaded {} cached prices from local file", prices.len());
    prices
}

/// Save prices to local cache file (called after successful remote fetch)
pub fn save_cached_prices(prices: &[ModelPricing]) {
    let path = match cache_path() {
        Some(p) => p,
        None => return,
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match serde_json::to_string_pretty(prices) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                log::warn!("Failed to save price cache: {}", e);
            } else {
                log::info!("Saved {} prices to local cache", prices.len());
            }
        }
        Err(e) => log::warn!("Failed to serialize price cache: {}", e),
    }
}

/// Merge prices with priority: custom > remote > cached
/// Returns the final merged price list
pub fn merge_prices(
    cached: &[ModelPricing],
    remote: &[ModelPricing],
    custom: &[ModelPricing],
) -> Vec<ModelPricing> {
    use std::collections::{HashMap, HashSet};

    let mut map: HashMap<String, ModelPricing> = HashMap::new();
    let mut has_default_set: HashSet<String> = HashSet::new();

    // Layer 1: cached (from last successful remote fetch)
    for p in cached {
        let mut price = p.clone();
        price.price_source = "cached".to_string();
        price.has_default = false;
        map.insert(p.model_id.clone(), price);
        has_default_set.insert(p.model_id.clone());
    }

    // Layer 2: remote (overrides cached)
    for p in remote {
        let mut price = p.clone();
        price.price_source = "remote".to_string();
        map.insert(p.model_id.clone(), price);
        has_default_set.insert(p.model_id.clone());
    }

    // Layer 3: custom (highest priority, user-defined)
    for p in custom {
        let mut price = p.clone();
        price.price_source = "custom".to_string();
        price.has_default = has_default_set.contains(&p.model_id);
        map.insert(p.model_id.clone(), price);
    }

    let mut result: Vec<ModelPricing> = map.into_values().collect();
    result.sort_by(|a, b| {
        // Custom prices first (newest by created_at, then model_id), then others alphabetically
        match (&a.created_at, &b.created_at) {
            (Some(_), None) => std::cmp::Ordering::Less,  // custom before non-custom
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(ta), Some(tb)) => {
                // Both custom: newest first, then model_id for ties
                tb.cmp(ta).then_with(|| a.model_id.cmp(&b.model_id))
            }
            (None, None) => a.model_id.cmp(&b.model_id), // Both non-custom: alphabetical
        }
    });
    result
}
