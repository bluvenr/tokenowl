use tauri::State;

use crate::commands::usage::DbState;
use crate::collectors::{CollectorManager, SourceStatus};
use crate::models::usage::MissingModelPrice;

#[tauri::command]
pub fn rescan(db: State<'_, DbState>) -> Result<u64, String> {
    let manager = CollectorManager::new(db.inner().clone());
    manager.rescan().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_source_status(db: State<'_, DbState>) -> Result<Vec<SourceStatus>, String> {
    let manager = CollectorManager::new(db.inner().clone());
    Ok(manager.get_source_status())
}

#[tauri::command]
pub fn get_models_without_prices(db: State<'_, DbState>) -> Result<Vec<MissingModelPrice>, String> {
    db.inner().get_models_without_prices().map_err(|e| e.to_string())
}
