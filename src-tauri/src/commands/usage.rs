use std::sync::Arc;
use tauri::State;
use crate::storage::database::Database;
use crate::storage::queries;
use crate::models::usage::*;

#[tauri::command]
pub async fn get_usage_summary(
    db: State<'_, Arc<Database>>,
    period: String,
) -> Result<UsageSummary, String> {
    queries::get_usage_summary(&db, &period).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_usage_by_model(
    db: State<'_, Arc<Database>>,
    period: String,
) -> Result<Vec<ModelUsage>, String> {
    queries::get_usage_by_model(&db, &period).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_usage_by_provider(
    db: State<'_, Arc<Database>>,
    period: String,
) -> Result<Vec<ProviderUsage>, String> {
    queries::get_usage_by_provider(&db, &period).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_usage_trend(
    db: State<'_, Arc<Database>>,
    granularity: String,
    period: String,
) -> Result<Vec<TrendPoint>, String> {
    queries::get_usage_trend(&db, &granularity, &period).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recent_sessions(
    db: State<'_, Arc<Database>>,
    limit: Option<u32>,
) -> Result<Vec<SessionSummary>, String> {
    queries::get_recent_sessions(&db, limit.unwrap_or(20)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_period_comparison(
    db: State<'_, Arc<Database>>,
    period: String,
) -> Result<PeriodComparison, String> {
    queries::get_period_comparison(&db, &period).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_cost_anomalies(
    db: State<'_, Arc<Database>>,
    period: String,
) -> Result<CostAnomalyReport, String> {
    queries::get_cost_anomalies(&db, &period).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_cost_attribution(
    db: State<'_, Arc<Database>>,
    period: String,
) -> Result<Vec<ProviderAttribution>, String> {
    queries::get_cost_attribution(&db, &period).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_budget_burn_rate(
    db: State<'_, Arc<Database>>,
) -> Result<BudgetBurnRate, String> {
    queries::get_budget_burn_rate(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_cache_trend(
    db: State<'_, Arc<Database>>,
    granularity: String,
    period: String,
) -> Result<Vec<CacheTrendPoint>, String> {
    queries::get_cache_trend(&db, &granularity, &period).map_err(|e| e.to_string())
}
