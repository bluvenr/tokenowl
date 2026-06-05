use std::sync::Arc;
use tauri::State;

use crate::ccswitch::syncer::CcSwitchSyncer;
use crate::models::usage::{CcSwitchStatus, SyncResult};

/// Shared syncer state managed by Tauri
pub type CcSwitchSyncerState = Arc<CcSwitchSyncer>;

/// Get CC Switch connection and sync status
#[tauri::command]
pub fn get_ccswitch_status(syncer: State<'_, CcSwitchSyncerState>) -> Result<CcSwitchStatus, String> {
    syncer.get_status().map_err(|e| e.to_string())
}

/// Manually trigger a sync cycle
#[tauri::command]
pub fn sync_ccswitch(syncer: State<'_, CcSwitchSyncerState>) -> Result<SyncResult, String> {
    syncer.sync().map_err(|e| e.to_string())
}

/// Get the CC Switch database path
#[tauri::command]
pub fn get_ccswitch_db_path(syncer: State<'_, CcSwitchSyncerState>) -> Result<String, String> {
    Ok(syncer.db_path().to_string_lossy().to_string())
}
