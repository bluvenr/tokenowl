use std::sync::Arc;
use tauri::State;

use crate::models::usage::*;
use crate::storage::database::Database;

pub type DbState = Arc<Database>;

#[tauri::command]
pub fn get_usage_summary(db: State<'_, DbState>, period: String) -> Result<UsageSummary, String> {
    db.get_usage_summary(&period).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_usage_by_model(db: State<'_, DbState>, period: String) -> Result<Vec<ModelUsage>, String> {
    db.get_usage_by_model(&period).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_usage_trend(
    db: State<'_, DbState>,
    granularity: String,
    period: String,
) -> Result<Vec<TrendPoint>, String> {
    db.get_usage_trend(&granularity, &period).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_recent_sessions(db: State<'_, DbState>, limit: u32) -> Result<Vec<SessionSummary>, String> {
    db.get_recent_sessions(limit).map_err(|e| e.to_string())
}
