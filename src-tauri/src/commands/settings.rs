use tauri::State;

use crate::commands::usage::DbState;
use crate::commands::remote::RemoteStateManaged;
use crate::collectors::CollectorManager;
use crate::models::settings::{AppSettings, ModelPricing, SourceConfig};
use crate::remote::download_source::update_shared;

#[tauri::command]
pub fn get_settings(db: State<'_, DbState>) -> Result<AppSettings, String> {
    db.get_app_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_settings(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    remote: State<'_, RemoteStateManaged>,
    settings: AppSettings,
) -> Result<(), String> {
    // Read current settings to detect if auto_start actually changed
    let old_settings = db.get_app_settings().unwrap_or_default();
    db.update_app_settings(&settings).map_err(|e| e.to_string())?;

    // Update download source in remote services if it changed
    if settings.download_source != old_settings.download_source {
        update_shared(&remote.download_source, &settings.download_source);
    }

    // Only sync auto-start with OS when the value actually changed
    if settings.auto_start != old_settings.auto_start {
        use tauri_plugin_autostart::ManagerExt;
        let autostart = app.autolaunch();
        match autostart.is_enabled() {
            Ok(current) if current == settings.auto_start => {
                log::info!("Auto-start already in desired state: {}", settings.auto_start);
            }
            _ => {
                if settings.auto_start {
                    match autostart.enable() {
                        Ok(()) => log::info!("Auto-start enabled via OS"),
                        Err(e) => log::warn!("Could not enable auto-start (may be dev mode): {}", e),
                    }
                } else {
                    match autostart.disable() {
                        Ok(()) => log::info!("Auto-start disabled via OS"),
                        Err(e) => log::warn!("Could not disable auto-start (may be dev mode): {}", e),
                    }
                }
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_source_configs(db: State<'_, DbState>) -> Result<Vec<SourceConfig>, String> {
    db.get_source_configs().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_source_config(
    db: State<'_, DbState>,
    source: String,
    enabled: bool,
    custom_path: Option<String>,
) -> Result<(), String> {
    db.update_source_config(&source, enabled, custom_path.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_custom_prices(db: State<'_, DbState>) -> Result<Vec<ModelPricing>, String> {
    db.get_custom_prices().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_custom_price(db: State<'_, DbState>, price: ModelPricing) -> Result<(), String> {
    db.upsert_custom_price(&price).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_custom_price(db: State<'_, DbState>, model_id: String) -> Result<(), String> {
    db.delete_custom_price(&model_id).map_err(|e| e.to_string())
}

/// Reset a custom price back to remote/cached defaults (same as delete)
#[tauri::command]
pub fn reset_custom_price(db: State<'_, DbState>, model_id: String) -> Result<(), String> {
    db.delete_custom_price(&model_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_prices(
    db: State<'_, DbState>,
    remote: State<'_, RemoteStateManaged>,
) -> Result<Vec<ModelPricing>, String> {
    let cached = crate::pricing::registry::load_cached_prices();
    let custom = db.get_custom_prices().map_err(|e| e.to_string())?;
    let remote_prices = remote.price_syncer.get_cached();
    let merged = crate::pricing::registry::merge_prices(&cached, &remote_prices, &custom);
    Ok(merged)
}

/// Recalculate cost_usd for all usage records of a specific model
/// using the current (updated) price. Returns number of affected records.
#[tauri::command]
pub fn recalculate_costs(db: State<'_, DbState>, model_id: String) -> Result<u64, String> {
    let manager = CollectorManager::new(db.inner().clone());
    manager.recalculate_model_costs(&model_id).map_err(|e| e.to_string())
}

/// Count usage records for a specific model (used to decide whether to show recalc dialog)
#[tauri::command]
pub fn count_model_records(db: State<'_, DbState>, model_id: String) -> Result<u64, String> {
    db.count_usage_records_for_model(&model_id).map_err(|e| e.to_string())
}
