use std::sync::Arc;
use tauri::{Manager, PhysicalPosition, Position};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use crate::TrayPositionState;
use crate::storage::database::Database;

/// Get translated menu labels for the given language
fn get_menu_labels(lang: &str) -> (&'static str, &'static str, &'static str) {
    if lang.starts_with("zh") {
        ("托盘浮窗", "设置", "退出")
    } else {
        ("Tray Popup", "Settings", "Quit")
    }
}

/// Build a tray context menu with the given language
fn build_tray_menu(
    app: &tauri::AppHandle,
    lang: &str,
) -> Result<tauri::menu::Menu<tauri::Wry>, tauri::Error> {
    let (popup_label, settings_label, quit_label) = get_menu_labels(lang);
    let popup_item = MenuItemBuilder::with_id("tray_popup", popup_label).build(app)?;
    let settings_item = MenuItemBuilder::with_id("settings", settings_label).build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", quit_label).build(app)?;
    MenuBuilder::new(app)
        .item(&popup_item)
        .item(&settings_item)
        .separator()
        .item(&quit_item)
        .build()
}

#[tauri::command]
pub async fn show_tray_popup(
    app: tauri::AppHandle,
    tray_state: tauri::State<'_, TrayPositionState>,
) -> Result<(), String> {
    let window = app
        .get_webview_window("tray")
        .ok_or_else(|| "Tray window not found".to_string())?;

    let pos = tray_state
        .last_position
        .lock()
        .ok()
        .and_then(|p| *p);

    if let Some(tray_pos) = pos {
        let win_height = 400;
        let x = tray_pos.x;
        let mut y = tray_pos.y - win_height;
        if y < 0 {
            y = tray_pos.y + 40;
        }
        window
            .set_position(Position::Physical(PhysicalPosition::new(x, y)))
            .map_err(|e| e.to_string())?;
    }

    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

/// Rebuild the tray context menu with labels matching the given language.
/// Called from the frontend when settings are loaded or language is changed.
#[tauri::command]
pub async fn update_tray_menu(
    app: tauri::AppHandle,
    language: String,
) -> Result<(), String> {
    // Resolve "auto" to actual language
    let lang = if language == "auto" {
        // Read from DB settings as fallback
        let db = app.state::<Arc<Database>>();
        let result = if let Ok(conn) = db.conn() {
            let stored: String = conn
                .query_row("SELECT language FROM app_settings WHERE id = 1", [], |r| r.get(0))
                .unwrap_or_else(|_| "en".to_string());
            if stored == "auto" {
                "en".to_string()
            } else {
                stored
            }
        } else {
            "en".to_string()
        };
        result
    } else {
        language
    };

    let menu = build_tray_menu(&app, &lang).map_err(|e| e.to_string())?;

    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
        log::info!("Tray menu updated for language: {}", lang);
    } else {
        log::warn!("Tray icon 'main' not found, cannot update menu");
    }

    Ok(())
}
