pub mod checker;
pub mod digest;

use std::sync::Arc;
use tauri::Emitter;
use crate::storage::database::Database;
use crate::storage::queries;
use checker::UpdateChecker;

const GITHUB_OWNER: &str = "bluvenr";
const GITHUB_REPO: &str = "tokenowl";

/// Handle for the update check scheduler task.
pub struct UpdateCheckHandle {
    pub thread_handle: std::thread::JoinHandle<()>,
}

/// Start the auto-update check scheduler.
/// Checks on startup (after 10s delay) and periodically based on `update_check_interval_hours`.
pub fn start_update_scheduler(
    db: Arc<Database>,
    app_handle: tauri::AppHandle,
) -> UpdateCheckHandle {
    log::info!("Starting auto-update check scheduler");

    let handle = std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            update_check_loop(db, app_handle);
        }));
        if let Err(e) = result {
            log::error!("Update check scheduler thread panicked: {:?}", e);
        }
    });

    UpdateCheckHandle {
        thread_handle: handle,
    }
}

fn update_check_loop(db: Arc<Database>, app_handle: tauri::AppHandle) {
    // Initial check after 10s delay (let the app settle)
    std::thread::sleep(std::time::Duration::from_secs(10));

    // Reuse a single runtime for the entire lifetime of the scheduler
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("Failed to create tokio runtime for update scheduler: {}", e);
            return;
        }
    };

    loop {
        // Read interval from settings
        let interval_hours = queries::get_app_settings(&db)
            .map(|s| s.update_check_interval_hours)
            .unwrap_or(4);

        // Perform the check
        let current_version = env!("CARGO_PKG_VERSION");
        let checker = UpdateChecker::new(GITHUB_OWNER, GITHUB_REPO, current_version);

        if let Ok(info) = rt.block_on(checker.check()) {
            if info.update_available {
                log::info!(
                    "Update available: {} -> {}",
                    info.current_version,
                    info.latest_version
                );
                // Emit event to frontend
                let _ = app_handle.emit("update-available", &serde_json::json!({
                    "current": info.current_version,
                    "latest": info.latest_version,
                    "release_url": info.release_url,
                    "changelog": info.changelog,
                }));
            } else {
                log::debug!("No update available (current: {})", info.current_version);
            }
        }

        // Sleep until next check
        let interval_secs = (interval_hours as u64) * 3600;
        // Check every 60s if we've passed the interval (more responsive to setting changes)
        let mut elapsed: u64 = 0;
        while elapsed < interval_secs {
            std::thread::sleep(std::time::Duration::from_secs(60));
            elapsed += 60;
        }
    }
}
