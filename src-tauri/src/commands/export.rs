use std::sync::Arc;
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_dialog::FilePath;
use crate::storage::database::Database;
use crate::storage::queries;

#[tauri::command]
pub async fn export_usage_csv(
    app: tauri::AppHandle,
    db: State<'_, Arc<Database>>,
    period: String,
) -> Result<(), String> {
    let csv = queries::export_csv(&db, &period).map_err(|e| e.to_string())?;

    // Show save dialog
    let file_path = app.dialog()
        .file()
        .set_file_name("tokenowl_export.csv")
        .blocking_save_file();

    if let Some(fp) = file_path {
        if let FilePath::Path(p) = fp {
            std::fs::write(p, csv).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn export_usage_json(
    app: tauri::AppHandle,
    db: State<'_, Arc<Database>>,
    period: String,
) -> Result<(), String> {
    let json = queries::export_json(&db, &period).map_err(|e| e.to_string())?;

    // Show save dialog
    let file_path = app.dialog()
        .file()
        .set_file_name("tokenowl_export.json")
        .blocking_save_file();

    if let Some(fp) = file_path {
        if let FilePath::Path(p) = fp {
            std::fs::write(p, json).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
