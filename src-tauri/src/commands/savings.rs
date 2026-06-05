use std::collections::HashMap;
use tauri::State;

use crate::models::usage::SavingsAnalysis;
use crate::commands::usage::DbState;
use crate::pricing::registry::{load_cached_prices, merge_prices};

/// Get complete savings analysis: cache efficiency, model spending patterns,
/// cost forecast, and anomaly detection — all in a single call.
#[tauri::command]
pub fn get_savings_analysis(
    db: State<'_, DbState>,
    period: String,
) -> Result<SavingsAnalysis, String> {
    // Build price map for accurate cache savings calculation
    let cached_prices = load_cached_prices();
    let custom_prices = db.get_custom_prices().unwrap_or_default();
    let merged = merge_prices(&cached_prices, &[], &custom_prices);
    let price_map: HashMap<String, &crate::models::settings::ModelPricing> = merged
        .iter()
        .map(|p| (p.model_id.clone(), p))
        .collect();

    // Compute cache savings using actual model prices
    let cache_savings = db
        .get_cache_savings_by_source(&period, &price_map)
        .unwrap_or_default();

    let cache_efficiency = db
        .get_cache_efficiency(&period, &cache_savings)
        .map_err(|e| e.to_string())?;
    let model_analysis = db.get_model_usage_analysis(&period).map_err(|e| e.to_string())?;
    let forecast = db.get_cost_forecast().map_err(|e| e.to_string())?;
    let anomaly_report = db.get_cost_anomalies(30, 2.5).map_err(|e| e.to_string())?;

    Ok(SavingsAnalysis {
        cache_efficiency,
        model_analysis,
        forecast,
        anomaly_report,
    })
}
