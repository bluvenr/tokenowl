use tauri::State;

use crate::models::budget::{BudgetConfig, BudgetAlert};
use crate::commands::usage::DbState;

#[tauri::command]
pub fn get_budget_config(db: State<'_, DbState>) -> Result<BudgetConfig, String> {
    db.get_budget_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_budget_config(db: State<'_, DbState>, config: BudgetConfig) -> Result<(), String> {
    db.update_budget_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_budget_alert(db: State<'_, DbState>) -> Result<Option<BudgetAlert>, String> {
    db.check_budget_status().map_err(|e| e.to_string())
}

/// Send a system notification via OS notification center
#[tauri::command]
pub fn send_notification(app: tauri::AppHandle, title: String, body: String) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;
    app.notification()
        .builder()
        .title(&title)
        .body(&body)
        .show()
        .map_err(|e| e.to_string())
}

/// Get database statistics for the settings page
#[tauri::command]
pub fn get_db_stats(db: State<'_, DbState>) -> Result<DbStats, String> {
    let conn = db.conn().map_err(|e| e.to_string())?;

    let record_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM usage_records", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let source_count: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT source) FROM usage_records",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let session_count: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT source || '|' || session_id) FROM usage_records",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let db_size = db.get_db_file_size().unwrap_or(0);

    Ok(DbStats {
        record_count: record_count as u64,
        source_count: source_count as u64,
        session_count: session_count as u64,
        db_size_bytes: db_size,
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbStats {
    pub record_count: u64,
    pub source_count: u64,
    pub session_count: u64,
    pub db_size_bytes: u64,
}
