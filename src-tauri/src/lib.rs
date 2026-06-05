use std::sync::Arc;
use std::sync::Mutex;
use tauri::{Emitter, Manager, PhysicalPosition, Position, WindowEvent};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};

/// Shared state for tray icon position tracking
pub struct TrayPositionState {
    pub last_position: Mutex<Option<PhysicalPosition<i32>>>,
}


pub mod error;
pub mod models;
pub mod storage;
pub mod ccswitch;
pub mod pricing;
pub mod commands;
pub mod remote;
pub mod updater;
pub mod crash;

use storage::database::Database;
use pricing::registry::PriceRegistry;
use ccswitch::syncer::{CcSwitchSyncerState, start_background_sync};
use ccswitch::syncer::CcSwitchSyncer;

/// GitHub repository coordinates for remote services
#[allow(dead_code)]
const GITHUB_OWNER: &str = "bluvenr";
#[allow(dead_code)]
const GITHUB_REPO: &str = "tokenowl";

/// Application data directory name
pub const APP_DATA_DIR: &str = "com.virapi.tokenowl";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("TokenOwl v{} starting up...", env!("CARGO_PKG_VERSION"));

    // Initialize database
    let db = Arc::new(Database::new().expect("Failed to initialize database"));
    log::info!("Database initialized");

    // Initialize price registry
    let price_registry = Arc::new(PriceRegistry::new().expect("Failed to initialize price registry"));
    
    // Load custom prices from database into registry
    if let Ok(custom_prices) = storage::queries::get_custom_prices(&db) {
        price_registry.load_custom_prices(custom_prices);
        log::info!("Custom prices loaded from database");
    }
    log::info!("Price registry initialized");

    // Initialize CC Switch syncer
    let ccswitch_syncer: CcSwitchSyncerState = Arc::new(std::sync::Mutex::new(
        CcSwitchSyncer::new(db.clone())
    ));
    log::info!("CC Switch syncer initialized");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(db.clone())
        .manage(price_registry.clone())
        .manage(ccswitch_syncer.clone())
        .manage(TrayPositionState { last_position: Mutex::new(None) })
        .setup(move |app| {
            // Register autostart plugin
            app.handle().plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                None,
            )).ok();

            // Start background auto-sync
            let sync_handle = start_background_sync(ccswitch_syncer.clone(), db.clone());
            app.manage(sync_handle);
            log::info!("Background sync started");

            // Start digest notification scheduler
            let digest_handle = updater::digest::start_digest_scheduler(db.clone(), app.handle().clone());
            app.manage(digest_handle);
            log::info!("Digest notification scheduler started");

            // Start auto-update check scheduler
            let update_handle = updater::start_update_scheduler(db.clone(), app.handle().clone());
            app.manage(update_handle);
            log::info!("Auto-update check scheduler started");

            // Build right-click context menu (English default; updated on settings load)
            let tray_popup_item = MenuItemBuilder::with_id("tray_popup", "Tray Popup").build(app)?;
            let settings_item = MenuItemBuilder::with_id("settings", "Settings").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&tray_popup_item)
                .item(&settings_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // Create tray icon
            let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))
                .expect("failed to load tray icon");
            let _tray = TrayIconBuilder::with_id("main")
                .icon(tray_icon)
                .tooltip("TokenOwl")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        position,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        // Store tray position for later use by menu items
                        if let Some(state) = app.try_state::<TrayPositionState>() {
                            if let Ok(mut pos) = state.last_position.lock() {
                                *pos = Some(PhysicalPosition::new(position.x as i32, position.y as i32));
                            }
                        }
                        // Left-click: show and focus the main window (bring to front even if already open)
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "tray_popup" => {
                        if let Some(window) = app.get_webview_window("tray") {
                            let pos = app.try_state::<TrayPositionState>()
                                .and_then(|s| s.last_position.lock().ok().and_then(|p| *p));
                            if let Some(tray_pos) = pos {
                                let win_height = 400;
                                let x = tray_pos.x;
                                let mut y = tray_pos.y - win_height;
                                if y < 0 { y = tray_pos.y + 40; }
                                let _ = window.set_position(Position::Physical(PhysicalPosition::new(x, y)));
                            }
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        let _ = app.emit("navigate", "settings");
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // Set window icon
            let win_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))
                .expect("failed to load window icon");
            if let Some(main_win) = app.get_webview_window("main") {
                let _ = main_win.set_icon(win_icon.clone());
            }
            if let Some(tray_win) = app.get_webview_window("tray") {
                let _ = tray_win.set_icon(win_icon);
            }

            // Intercept main window close: hide to tray instead of exiting
            if let Some(main_win) = app.get_webview_window("main") {
                let main_win_for_close = main_win.clone();
                main_win.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = main_win_for_close.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Usage commands
            commands::usage::get_usage_summary,
            commands::usage::get_usage_by_model,
            commands::usage::get_usage_by_provider,
            commands::usage::get_usage_trend,
            commands::usage::get_recent_sessions,
            commands::usage::get_period_comparison,
            commands::usage::get_cost_anomalies,
            commands::usage::get_cost_attribution,
            commands::usage::get_budget_burn_rate,
            commands::usage::get_cache_trend,
            // Budget commands
            commands::budget::get_budget_config,
            commands::budget::update_budget_config,
            commands::budget::check_budget_alert,
            commands::budget::send_notification,
            commands::budget::get_db_stats,
            // Export commands
            commands::export::export_usage_csv,
            commands::export::export_usage_json,
            // Settings commands
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::set_autostart,
            commands::settings::is_autostart_enabled,
            commands::settings::get_custom_prices,
            commands::settings::update_custom_price,
            commands::settings::delete_custom_price,
            commands::settings::reset_custom_price,
            commands::settings::get_all_prices,
            commands::settings::count_model_records,
            commands::settings::get_models_without_prices,
            commands::settings::quit_app,
            // CC Switch commands
            commands::ccswitch::get_ccswitch_status,
            commands::ccswitch::sync_ccswitch,
            commands::ccswitch::get_ccswitch_db_path,
            commands::ccswitch::ccswitch_update_sync_config,
            commands::ccswitch::ccswitch_set_db_path,
            // Savings commands
            commands::savings::get_savings_analysis,
            // Remote commands
            commands::remote::get_app_version,
            commands::remote::check_for_update,
            commands::remote::fetch_remote_config,
            commands::remote::get_crash_logs,
            commands::remote::delete_crash_log,
            commands::remote::clear_crash_logs,
            commands::remote::get_crash_issue_url,
            // Tray commands
            commands::tray::show_tray_popup,
            commands::tray::update_tray_menu,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
