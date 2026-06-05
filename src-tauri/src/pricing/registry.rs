use std::collections::HashMap;
use std::sync::RwLock;
use crate::models::settings::{ModelPricing, PriceSource, CustomPrice};
use crate::error::AppResult;

/// Built-in model prices embedded in the binary
const BUILTIN_PRICES_JSON: &str = include_str!("../data/model_prices.json");

/// Price registry managing built-in, remote, and custom prices
pub struct PriceRegistry {
    builtin_prices: HashMap<String, ModelPricing>,
    remote_prices: RwLock<HashMap<String, ModelPricing>>,
    custom_prices: RwLock<HashMap<String, CustomPrice>>,
}

impl PriceRegistry {
    /// Create a new price registry with built-in prices
    pub fn new() -> AppResult<Self> {
        let builtin: Vec<ModelPricing> = serde_json::from_str(BUILTIN_PRICES_JSON)
            .unwrap_or_default();

        let builtin_prices: HashMap<String, ModelPricing> = builtin
            .into_iter()
            .map(|p| (p.model_id.clone(), p))
            .collect();

        Ok(Self {
            builtin_prices,
            remote_prices: RwLock::new(HashMap::new()),
            custom_prices: RwLock::new(HashMap::new()),
        })
    }

    /// Get price for a model (priority: custom > remote > builtin)
    pub fn get_price(&self, model_id: &str) -> Option<ModelPricing> {
        // Check custom prices first
        if let Ok(custom) = self.custom_prices.read() {
            if let Some(cp) = custom.get(model_id) {
                return Some(ModelPricing {
                    model_id: cp.model_id.clone(),
                    display_name: cp.model_id.clone(),
                    input_per_million: cp.input_per_million,
                    output_per_million: cp.output_per_million,
                    cache_write_per_million: cp.cache_write_per_million,
                    cache_read_per_million: cp.cache_read_per_million,
                    source: PriceSource::Custom,
                });
            }
        }

        // Check remote prices
        if let Ok(remote) = self.remote_prices.read() {
            if let Some(p) = remote.get(model_id) {
                return Some(p.clone());
            }
        }

        // Fall back to builtin prices
        self.builtin_prices.get(model_id).cloned()
    }

    /// Get all available prices
    pub fn get_all_prices(&self) -> Vec<ModelPricing> {
        let mut result: HashMap<String, ModelPricing> = self.builtin_prices.clone();

        // Merge remote prices
        if let Ok(remote) = self.remote_prices.read() {
            for (id, price) in remote.iter() {
                result.insert(id.clone(), price.clone());
            }
        }

        // Override with custom prices
        if let Ok(custom) = self.custom_prices.read() {
            for (id, cp) in custom.iter() {
                result.insert(id.clone(), ModelPricing {
                    model_id: cp.model_id.clone(),
                    display_name: cp.model_id.clone(),
                    input_per_million: cp.input_per_million,
                    output_per_million: cp.output_per_million,
                    cache_write_per_million: cp.cache_write_per_million,
                    cache_read_per_million: cp.cache_read_per_million,
                    source: PriceSource::Custom,
                });
            }
        }

        result.into_values().collect()
    }

    /// Update remote prices
    pub fn update_remote_prices(&self, prices: Vec<ModelPricing>) {
        if let Ok(mut remote) = self.remote_prices.write() {
            *remote = prices.into_iter().map(|p| (p.model_id.clone(), p)).collect();
        }
    }

    /// Set a custom price
    pub fn set_custom_price(&self, price: CustomPrice) {
        if let Ok(mut custom) = self.custom_prices.write() {
            custom.insert(price.model_id.clone(), price);
        }
    }

    /// Remove a custom price
    pub fn remove_custom_price(&self, model_id: &str) {
        if let Ok(mut custom) = self.custom_prices.write() {
            custom.remove(model_id);
        }
    }

    /// Load custom prices from database
    pub fn load_custom_prices(&self, prices: Vec<CustomPrice>) {
        if let Ok(mut custom) = self.custom_prices.write() {
            *custom = prices.into_iter().map(|p| (p.model_id.clone(), p)).collect();
        }
    }
}
