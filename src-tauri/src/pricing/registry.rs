use crate::models::settings::ModelPricing;

/// Load built-in model prices from embedded JSON
pub fn load_builtin_prices() -> Vec<ModelPricing> {
    let json = include_str!("../../data/model_prices.json");
    let mut prices: Vec<ModelPricing> =
        serde_json::from_str(json).unwrap_or_else(|e| {
            log::error!("Failed to parse builtin prices: {}", e);
            vec![]
        });
    for p in &mut prices {
        p.price_source = "builtin".to_string();
    }
    prices
}

/// Merge prices with priority: custom > remote > builtin
/// Returns the final merged price list
pub fn merge_prices(
    builtin: &[ModelPricing],
    remote: &[ModelPricing],
    custom: &[ModelPricing],
) -> Vec<ModelPricing> {
    use std::collections::HashMap;

    let mut map: HashMap<String, ModelPricing> = HashMap::new();

    // Layer 1: builtin (lowest priority)
    for p in builtin {
        map.insert(p.model_id.clone(), p.clone());
    }

    // Layer 2: remote (overrides builtin)
    for p in remote {
        let mut price = p.clone();
        price.price_source = "remote".to_string();
        map.insert(p.model_id.clone(), price);
    }

    // Layer 3: custom (highest priority)
    for p in custom {
        let mut price = p.clone();
        price.price_source = "custom".to_string();
        map.insert(p.model_id.clone(), price);
    }

    map.into_values().collect()
}
