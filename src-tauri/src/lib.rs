use std::sync::Arc;
use tauri::{Emitter, Manager, PhysicalPosition, Position, WindowEvent};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};
use tauri::menu::{MenuBuilder, MenuItemBuilder};

pub mod error;
pub mod models;
pub mod storage;
pub mod collectors;
pub mod pricing;
pub mod watcher;
pub mod commands;
pub mod remote;
pub mod updater;
pub mod crash;

use storage::database::Database;
use collectors::CollectorManager;
use watcher::file_watcher::FileWatcher;
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

    // Create collector manager and run initial scan
    let manager = Arc::new(CollectorManager::new(db.clone()));
    if let Err(e) = manager.initial_scan() {
        log::error!("Initial scan failed: {}", e);

        // Log crash
        if let Some(logger) = crash::logger::CrashLogger::new() {
            logger.log_crash(&error::AppError::Config(format!("Initial scan failed: {}", e)));
        }
    }

    // Log source status
    for status in manager.get_source_status() {
        log::info!(
            "  Source [{}]: available={}, enabled={}",
            status.display_name, status.available, status.enabled
        );
    }

    // Set up file watcher
    let watch_paths = manager.all_watch_paths();
    let mut file_watcher = FileWatcher::new();
    let watcher_active = if !watch_paths.is_empty() {
        if let Err(e) = file_watcher.watch_paths(&watch_paths) {
            log::error!("Failed to start file watcher: {}", e);
            false
        } else {
            true
        }
    } else {
        log::info!("No watch paths available (no AI tools detected)");
        false
    };

    // Initialize remote services
    let app_settings = db.get_app_settings().unwrap_or_default();
    let remote_state = Arc::new(RemoteState::new(
        GITHUB_OWNER,
        GITHUB_REPO,
        app_settings.price_sync_interval_hours,
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
        .manage(db.clone())
        .manage(remote_state.clone())
        .setup(move |app| {
            // Register autostart plugin
            app.handle().plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                None,
            )).ok();

            // Build tray menu with translated text based on saved language
            let is_zh = app_settings.language.starts_with("zh");
            let open_text = if is_zh { "打开仪表盘" } else { "Open Dashboard" };
            let rescan_text = if is_zh { "重新扫描" } else { "Rescan Data" };
            let quit_text = if is_zh { "退出" } else { "Quit" };

            let open_item = MenuItemBuilder::with_id("open_dashboard", open_text).build(app)?;
            let rescan_item = MenuItemBuilder::with_id("rescan", rescan_text).build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", quit_text).build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&open_item)
                .item(&rescan_item)
                .separator()
                .item(&quit_item)
                .build()?;

            // Create tray icon with compile-time embedded icon
            let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/32x32.png"))
                .expect("failed to load tray icon");
            let _tray = TrayIconBuilder::with_id("main")
                .icon(tray_icon)
                .menu(&menu)
                .tooltip("TokenOwl - AI Cost Tracker")
                .on_tray_icon_event(|tray, event| {
                    // Left-click: toggle tray popup window, positioned near the tray icon
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
                                // Position window near the tray icon
                                let tray_x = position.x as i32;
                                let tray_y = position.y as i32;
                                let win_height = 400;
                                // Default: above the tray icon (taskbar at bottom)
                                let x = tray_x;
                                let mut y = tray_y - win_height;
                                // If above would go off-screen, show below instead
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
                    "rescan" => {
                        log::info!("Manual rescan triggered from tray");
                        let _ = app.emit("tokenowl:rescan-requested", ());
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

            // Start file watcher event loop in background thread
            let app_handle = app.handle().clone();
            if watcher_active {
                if let Err(e) = file_watcher.start_event_loop(manager, app_handle.clone()) {
                    log::error!("Failed to start watcher event loop: {}", e);
                } else {
                    // Keep file_watcher alive for the app's lifetime by storing it
                    // in Tauri's managed state. Without this, `file_watcher` is dropped
                    // at the end of setup(), which drops the RecommendedWatcher and its
                    // channel sender — causing the event loop thread to exit immediately.
                    app.manage(std::sync::Mutex::new(file_watcher));
                }
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
            let price_interval = app_settings.price_sync_interval_hours;

            // 1. Update checker (periodic background task)
            updater::checker::UpdateChecker::start_periodic_check(
                GITHUB_OWNER.to_string(),
                GITHUB_REPO.to_string(),
                env!("CARGO_PKG_VERSION").to_string(),
                update_interval,
                remote_state.download_source.clone(),
                app_handle.clone(),
            );

            // 2. Remote price syncer (periodic background task)
            let remote_for_sync = remote_state.clone();
            let app_for_sync = app_handle.clone();
            let db_for_sync = db.clone();
            if price_interval > 0 {
                tauri::async_runtime::spawn(async move {
                    // Initial sync after 10 seconds
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    let prices = remote_for_sync.price_syncer.sync_prices().await;
                    log::info!("Initial remote price sync: {} models", prices.len());

                    // Backfill costs with newly fetched remote prices
                    if !prices.is_empty() {
                        let manager = CollectorManager::new(db_for_sync.clone());
                        if let Ok(count) = manager.backfill_costs_with_remote(&prices) {
                            if count > 0 {
                                log::info!("Backfilled {} records with remote prices", count);
                            }
                        }

                        use tauri::Emitter;
                        let _ = app_for_sync.emit("tokenowl:prices-synced", prices.len());
                    }

                    // Periodic sync
                    let interval = std::time::Duration::from_secs(price_interval as u64 * 3600);
                    loop {
                        tokio::time::sleep(interval).await;
                        let p = remote_for_sync.price_syncer.sync_prices().await;
                        log::info!("Periodic price sync: {} models", p.len());
                        if !p.is_empty() {
                            let mgr = CollectorManager::new(db_for_sync.clone());
                            let _ = mgr.backfill_costs_with_remote(&p);
                        }
                    }
                });
            }

            // 3. Remote config fetcher (one-shot on startup)
            let remote_for_config = remote_state.clone();
            let app_for_config = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                if let Some(config) = remote_for_config.config_manager.fetch_config().await {
                    log::info!("Remote config loaded");
                    // Check for announcement
                    if let Some(announcement) = &config.announcement {
                        use tauri::Emitter;
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
            commands::usage::get_usage_by_source,
            commands::usage::get_usage_by_model,
            commands::usage::get_usage_trend,
            commands::usage::get_recent_sessions,
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
            commands::settings::get_source_configs,
            commands::settings::update_source_config,
            commands::settings::get_custom_prices,
            commands::settings::update_custom_price,
            commands::settings::delete_custom_price,
            commands::settings::reset_custom_price,
            commands::settings::get_all_prices,
            commands::settings::recalculate_costs,
            commands::settings::count_model_records,
            // Scan
            commands::scan::rescan,
            commands::scan::get_source_status,
            commands::scan::get_models_without_prices,
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
    rescan_text: String,
    quit_text: String,
) -> Result<(), String> {
    let menu = MenuBuilder::new(&app)
        .item(&MenuItemBuilder::with_id("open_dashboard", open_text).build(&app).map_err(|e| e.to_string())?)
        .item(&MenuItemBuilder::with_id("rescan", rescan_text).build(&app).map_err(|e| e.to_string())?)
        .separator()
        .item(&MenuItemBuilder::with_id("quit", quit_text).build(&app).map_err(|e| e.to_string())?)
        .build()
        .map_err(|e| e.to_string())?;

    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
    }
    Ok(())
}
