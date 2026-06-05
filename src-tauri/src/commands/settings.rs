use std::sync::Arc;
use tauri::State;
use crate::storage::database::Database;
use crate::storage::queries;
use crate::models::settings::*;
use crate::pricing::registry::PriceRegistry;

#[tauri::command]
pub async fn get_settings(
    db: State<'_, Arc<Database>>,
) -> Result<AppSettings, String> {
    queries::get_app_settings(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_settings(
    db: State<'_, Arc<Database>>,
    settings: AppSettings,
) -> Result<(), String> {
    queries::update_app_settings(&db, &settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_autostart(
    app: tauri::AppHandle,
    enabled: bool,
) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    
    let autolaunch = app.autolaunch();
    if enabled {
        autolaunch.enable().map_err(|e| e.to_string())?;
    } else {
        autolaunch.disable().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn is_autostart_enabled(
    app: tauri::AppHandle,
) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    
    let autolaunch = app.autolaunch();
    autolaunch.is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_custom_prices(
    db: State<'_, Arc<Database>>,
) -> Result<Vec<CustomPrice>, String> {
    queries::get_custom_prices(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_custom_price(
    db: State<'_, Arc<Database>>,
    registry: State<'_, Arc<PriceRegistry>>,
    price: CustomPrice,
) -> Result<(), String> {
    queries::upsert_custom_price(&db, &price).map_err(|e| e.to_string())?;
    registry.set_custom_price(price);
    Ok(())
}

#[tauri::command]
pub async fn delete_custom_price(
    db: State<'_, Arc<Database>>,
    registry: State<'_, Arc<PriceRegistry>>,
    model_id: String,
) -> Result<(), String> {
    queries::delete_custom_price(&db, &model_id).map_err(|e| e.to_string())?;
    registry.remove_custom_price(&model_id);
    Ok(())
}

#[tauri::command]
pub async fn reset_custom_price(
    db: State<'_, Arc<Database>>,
    registry: State<'_, Arc<PriceRegistry>>,
    model_id: String,
) -> Result<(), String> {
    queries::delete_custom_price(&db, &model_id).map_err(|e| e.to_string())?;
    registry.remove_custom_price(&model_id);
    Ok(())
}

#[tauri::command]
pub async fn get_all_prices(
    registry: State<'_, Arc<PriceRegistry>>,
) -> Result<Vec<ModelPricing>, String> {
    Ok(registry.get_all_prices())
}

#[tauri::command]
pub async fn count_model_records(
    db: State<'_, Arc<Database>>,
) -> Result<Vec<(String, u64)>, String> {
    queries::count_model_records(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_models_without_prices(
    db: State<'_, Arc<Database>>,
) -> Result<Vec<String>, String> {
    queries::get_models_without_prices(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}
