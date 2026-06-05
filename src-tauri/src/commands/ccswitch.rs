use std::sync::Arc;
use tauri::State;
use serde::{Deserialize, Serialize};
use crate::ccswitch::syncer::{CcSwitchSyncerState, SyncResult};
use crate::storage::database::Database;

/// CC Switch status for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcSwitchStatus {
    pub detected: bool,
    pub db_path: Option<String>,
    pub custom_db_path: Option<String>,
    pub proxy_running: bool,
    pub total_records: u64,
    pub last_sync_time: Option<String>,
    pub sync_interval_secs: u64,
    pub provider_count: u64,
    pub success_rate: f64,
}

#[tauri::command]
pub async fn get_ccswitch_status(
    syncer: State<'_, CcSwitchSyncerState>,
    db: State<'_, Arc<Database>>,
) -> Result<CcSwitchStatus, String> {
    let s = syncer.lock().map_err(|e| e.to_string())?;

    let detected = s.is_detected();
    let db_path = s.db_path().map(|p| p.to_string_lossy().to_string());

    // Try to get proxy running status and total count
    let (proxy_running, total_records) = if detected {
        match crate::ccswitch::reader::CcSwitchReader::detect() {
            Ok(path) => {
                let reader = crate::ccswitch::reader::CcSwitchReader::new(path);
                let running = reader.is_proxy_running();
                let count = reader.total_count().unwrap_or(0);
                (running, count)
            }
            Err(_) => (false, 0),
        }
    } else {
        (false, 0)
    };

    // Get last sync time from database
    let last_sync_time = s.last_sync_time();

    // Read sync config from sync_state table
    let (custom_db_path, sync_interval_secs) = match db.conn() {
        Ok(conn) => {
            let custom_db: Option<String> = conn.query_row(
                "SELECT cc_switch_db_path FROM sync_state WHERE id = 1", [], |r| r.get(0),
            ).ok().flatten();
            let interval: u64 = conn.query_row(
                "SELECT sync_interval_secs FROM sync_state WHERE id = 1", [], |r| r.get(0),
            ).unwrap_or(300);
            (custom_db, interval)
        }
        Err(_) => (None, 300),
    };

    // Compute provider count and success rate from usage_records
    let (provider_count, success_rate) = match db.conn() {
        Ok(conn) => {
            let pc: u64 = conn.query_row(
                "SELECT COUNT(DISTINCT COALESCE(provider_name, 'unknown')) FROM usage_records",
                [], |r| r.get(0),
            ).unwrap_or(0);

            let sr: f64 = conn.query_row(
                "SELECT
                    CASE WHEN COUNT(*) = 0 THEN 100.0
                    ELSE COALESCE(SUM(CASE WHEN status_code IS NULL OR status_code < 400 THEN 1 ELSE 0 END) * 100.0 / COUNT(*), 100.0)
                    END
                FROM usage_records",
                [], |r| r.get(0),
            ).unwrap_or(100.0);

            (pc, sr)
        }
        Err(_) => (0, 100.0),
    };

    Ok(CcSwitchStatus {
        detected,
        db_path,
        custom_db_path,
        proxy_running,
        total_records,
        last_sync_time,
        sync_interval_secs,
        provider_count,
        success_rate,
    })
}

#[tauri::command]
pub async fn sync_ccswitch(
    syncer: State<'_, CcSwitchSyncerState>,
) -> Result<SyncResult, String> {
    // Clone the Arc to avoid holding the Tauri State lock during the blocking sync.
    // This prevents potential deadlocks with other concurrent Tauri commands.
    let syncer = syncer.inner().clone();

    // Run blocking sync on a separate thread pool to avoid starving
    // the Tauri async runtime
    tokio::task::spawn_blocking(move || {
        let mut s = syncer.lock().map_err(|e| e.to_string())?;
        s.sync().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn get_ccswitch_db_path(
    syncer: State<'_, CcSwitchSyncerState>,
) -> Result<Option<String>, String> {
    let s = syncer.lock().map_err(|e| e.to_string())?;
    Ok(s.db_path().map(|p| p.to_string_lossy().to_string()))
}

/// Update the sync interval (seconds). 0 means disable auto-sync.
#[tauri::command]
pub async fn ccswitch_update_sync_config(
    db: State<'_, Arc<Database>>,
    sync_interval_secs: u64,
) -> Result<(), String> {
    let conn = db.conn().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE sync_state SET sync_interval_secs = ?1 WHERE id = 1",
        rusqlite::params![sync_interval_secs],
    )
    .map_err(|e| e.to_string())?;
    log::info!("Sync interval updated to {}s", sync_interval_secs);
    Ok(())
}

/// Set a custom CC Switch database path (or clear it to use auto-detect).
#[tauri::command]
pub async fn ccswitch_set_db_path(
    db: State<'_, Arc<Database>>,
    path: Option<String>,
) -> Result<(), String> {
    // Validate path if provided
    if let Some(ref p) = path {
        if !p.is_empty() {
            let pb = std::path::PathBuf::from(p);
            if !pb.exists() {
                return Err(format!("Path does not exist: {}", p));
            }
            if !pb.is_file() {
                return Err(format!("Path is not a file: {}", p));
            }
        }
    }

    let conn = db.conn().map_err(|e| e.to_string())?;
    let val = path.as_deref().filter(|s| !s.is_empty());
    conn.execute(
        "UPDATE sync_state SET cc_switch_db_path = ?1 WHERE id = 1",
        rusqlite::params![val],
    )
    .map_err(|e| e.to_string())?;
    log::info!("Custom CC Switch DB path updated: {:?}", val);
    Ok(())
}
