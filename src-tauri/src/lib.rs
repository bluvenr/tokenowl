use std::sync::Arc;
use tauri::{Emitter, Manager, PhysicalPosition, Position, WindowEvent};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};
use tauri::menu::{MenuBuilder, MenuItemBuilder};

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
use ccswitch::syncer::CcSwitchSyncer;
use commands::remote::RemoteState;

/// GitHub repository coordinates for remote services
const GITHUB_OWNER: &str = "bluvenr";
const GITHUB_REPO: &str = "tokenowl";

/// Application data directory name (matches tauri.conf.json identifier)
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

    // Initialize CC Switch syncer
    let syncer = Arc::new(CcSwitchSyncer::new(db.clone()));
    if syncer.is_detected() {
        log::info!("CC Switch detected at: {}", syncer.db_path().display());

        // Run initial sync
        match syncer.sync() {
            Ok(result) => {
                log::info!(
                    "Initial CC Switch sync: {} new records ({}ms)",
                    result.new_records, result.sync_duration_ms
                );
            }
            Err(e) => {
                log::error!("Initial CC Switch sync failed: {}", e);
            }
        }
    } else {
        log::info!("CC Switch not detected (waiting for installation...)");
    }

    // Initialize remote services
    let app_settings = db.get_app_settings().unwrap_or_default();
    let remote_state = Arc::new(RemoteState::new(
        GITHUB_OWNER,
        GITHUB_REPO,
        &app_settings.download_source,
    ));

    // Set up panic hook for crash logging
    let crash_logger = crash::logger::CrashLogger::new();
    if let Some(logger) = crash_logger {
        let logger = Arc::new(logger);
        std::panic::set_hook(Box::new(move |info| {
            let msg = format!("{}", info);
            logger.log_panic(&msg);
        }));
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(db.clone())
        .manage(remote_state.clone())
        .manage(syncer.clone() as commands::ccswitch::CcSwitchSyncerState)
        .setup(move |app| {
            // Register autostart plugin
            app.handle().plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                None,
            )).ok();

            // Build tray menu with translated text based on saved language
            let is_zh = app_settings.language.starts_with("zh");
            let open_text = if is_zh { "打开仪表盘" } else { "Open Dashboard" };
            let sync_text = if is_zh { "同步数据" } else { "Sync Data" };
            let quit_text = if is_zh { "退出" } else { "Quit" };

            let open_item = MenuItemBuilder::with_id("open_dashboard", open_text).build(app)?;
            let sync_item = MenuItemBuilder::with_id("sync", sync_text).build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", quit_text).build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&open_item)
                .item(&sync_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // Create tray icon with compile-time embedded icon
            let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))
                .expect("failed to load tray icon");
            let _tray = TrayIconBuilder::with_id("main")
                .icon(tray_icon)
                .menu(&menu)
                .tooltip("TokenOwl - CC Switch 的数据分析搭档")
                .on_tray_icon_event(|tray, event| {
                    // Left-click: toggle tray popup window
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        position,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("tray") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let tray_x = position.x as i32;
                                let tray_y = position.y as i32;
                                let win_height = 400;
                                let x = tray_x;
                                let mut y = tray_y - win_height;
                                if y < 0 {
                                    y = tray_y + 40;
                                }
                                let _ = window.set_position(Position::Physical(
                                    PhysicalPosition::new(x, y),
                                ));
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "open_dashboard" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "sync" => {
                        log::info!("Manual sync triggered from tray");
                        let _ = app.emit("tokenowl:sync-requested", ());
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // Set window icon for main and tray windows (compile-time embedded)
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

            // Apply auto-start setting from saved preferences
            {
                use tauri_plugin_autostart::ManagerExt;
                let autostart = app.autolaunch();
                if app_settings.auto_start {
                    match autostart.enable() {
                        Ok(()) => log::info!("Auto-start enabled"),
                        Err(e) => log::warn!("Could not enable auto-start (dev mode?): {}", e),
                    }
                } else {
                    match autostart.disable() {
                        Ok(()) => log::info!("Auto-start disabled"),
                        Err(e) => log::warn!("Could not disable auto-start (dev mode?): {}", e),
                    }
                }
            }

            // Start background services
            let update_interval = app_settings.update_check_interval_hours;
            let app_handle = app.handle().clone();

            // 1. Update checker (periodic background task)
            updater::checker::UpdateChecker::start_periodic_check(
                GITHUB_OWNER.to_string(),
                GITHUB_REPO.to_string(),
                env!("CARGO_PKG_VERSION").to_string(),
                update_interval,
                remote_state.download_source.clone(),
                app_handle.clone(),
            );

            // 2. CC Switch periodic syncer (every 5 minutes)
            let syncer_for_sync = syncer.clone();
            tauri::async_runtime::spawn(async move {
                // Initial delay
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                let interval = std::time::Duration::from_secs(300); // 5 minutes
                loop {
                    if let Ok(result) = syncer_for_sync.sync() {
                        if result.new_records > 0 {
                            log::info!("Periodic sync: {} new records", result.new_records);
                        }
                    }
                    tokio::time::sleep(interval).await;
                }
            });

            // 3. Remote price syncer (one-shot on startup)
            let remote_for_prices = remote_state.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                let prices = remote_for_prices.price_syncer.force_sync().await;
                log::info!("Initial remote price sync: {} models", prices.len());
            });

            // 4. Remote config fetcher (one-shot on startup)
            let remote_for_config = remote_state.clone();
            let app_for_config = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                if let Some(config) = remote_for_config.config_manager.fetch_config().await {
                    log::info!("Remote config loaded");
                    if let Some(announcement) = &config.announcement {
                        let _ = app_for_config.emit("tokenowl:announcement", announcement);
                        log::info!("Announcement: {}", announcement.title);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Usage queries
            commands::usage::get_usage_summary,
            commands::usage::get_usage_by_model,
            commands::usage::get_usage_trend,
            commands::usage::get_recent_sessions,
            // CC Switch
            commands::ccswitch::get_ccswitch_status,
            commands::ccswitch::sync_ccswitch,
            commands::ccswitch::get_ccswitch_db_path,
            // Budget
            commands::budget::get_budget_config,
            commands::budget::update_budget_config,
            commands::budget::check_budget_alert,
            commands::budget::send_notification,
            commands::budget::get_db_stats,
            // Export
            commands::export::export_usage_csv,
            commands::export::export_usage_json,
            // Settings
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::get_custom_prices,
            commands::settings::update_custom_price,
            commands::settings::delete_custom_price,
            commands::settings::reset_custom_price,
            commands::settings::get_all_prices,
            commands::settings::count_model_records,
            // Savings Engine
            commands::savings::get_savings_analysis,
            // Remote services
            commands::remote::get_app_version,
            commands::remote::check_for_update,
            commands::remote::fetch_remote_config,
            commands::remote::sync_remote_prices,
            commands::remote::get_crash_logs,
            commands::remote::delete_crash_log,
            commands::remote::clear_crash_logs,
            commands::remote::get_crash_issue_url,
            // Tray
            rebuild_tray_menu,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Rebuild the tray menu with updated localized text (called from frontend when language changes)
#[tauri::command]
fn rebuild_tray_menu(
    app: tauri::AppHandle,
    open_text: String,
    sync_text: String,
    quit_text: String,
) -> Result<(), String> {
    let menu = MenuBuilder::new(&app)
        .item(&MenuItemBuilder::with_id("open_dashboard", open_text).build(&app).map_err(|e| e.to_string())?)
        .item(&MenuItemBuilder::with_id("sync", sync_text).build(&app).map_err(|e| e.to_string())?)
        .separator()
        .item(&MenuItemBuilder::with_id("quit", quit_text).build(&app).map_err(|e| e.to_string())?)
        .build()
        .map_err(|e| e.to_string())?;

    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
    }
    Ok(())
}
