use std::sync::Arc;
use tauri::State;
use serde::{Deserialize, Serialize};
use crate::storage::database::Database;
use crate::updater::checker::UpdateChecker;
use crate::remote::config::ConfigManager;

/// Application version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppVersion {
    pub current: String,
    pub latest: Option<String>,
    pub update_available: bool,
    pub release_url: Option<String>,
    pub changelog: Option<String>,
}

/// Crash log entry (re-export from crash::logger)
pub use crate::crash::logger::CrashLogEntry;

const GITHUB_OWNER: &str = "bluvenr";
const GITHUB_REPO: &str = "tokenowl";

#[tauri::command]
pub async fn get_app_version() -> Result<AppVersion, String> {
    Ok(AppVersion {
        current: env!("CARGO_PKG_VERSION").to_string(),
        latest: None,
        update_available: false,
        release_url: None,
        changelog: None,
    })
}

#[tauri::command]
pub async fn check_for_update() -> Result<AppVersion, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let checker = UpdateChecker::new(GITHUB_OWNER, GITHUB_REPO, &current_version);

    match checker.check().await {
        Ok(info) => Ok(AppVersion {
            current: info.current_version,
            latest: Some(info.latest_version),
            update_available: info.update_available,
            release_url: info.release_url,
            changelog: info.changelog,
        }),
        Err(e) => {
            log::warn!("Update check failed: {}", e);
            Ok(AppVersion {
                current: current_version,
                latest: None,
                update_available: false,
                release_url: None,
                changelog: None,
            })
        }
    }
}

#[tauri::command]
pub async fn fetch_remote_config() -> Result<serde_json::Value, String> {
    let mut manager = ConfigManager::new();
    match manager.fetch().await {
        Ok(config) => serde_json::to_value(&config).map_err(|e| e.to_string()),
        Err(e) => {
            log::warn!("Remote config fetch failed: {}", e);
            Ok(serde_json::json!({}))
        }
    }
}

#[tauri::command]
pub async fn get_crash_logs(
    db: State<'_, Arc<Database>>,
) -> Result<Vec<CrashLogEntry>, String> {
    let conn = db.conn().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, error_type, message, backtrace FROM crash_logs ORDER BY timestamp DESC LIMIT 50"
    ).map_err(|e| e.to_string())?;

    let entries: Vec<CrashLogEntry> = stmt
        .query_map([], |row| {
            Ok(CrashLogEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                error_type: row.get(2)?,
                message: row.get(3)?,
                backtrace: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(entries)
}

#[tauri::command]
pub async fn delete_crash_log(
    db: State<'_, Arc<Database>>,
    id: String,
) -> Result<(), String> {
    let conn = db.conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM crash_logs WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn clear_crash_logs(
    db: State<'_, Arc<Database>>,
) -> Result<(), String> {
    let conn = db.conn().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM crash_logs", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_crash_issue_url() -> Result<String, String> {
    Ok("https://github.com/bluvenr/tokenowl/issues/new?template=crash_report.md".to_string())
}
