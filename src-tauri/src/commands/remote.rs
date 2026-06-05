use std::sync::Arc;
use tauri::State;

use crate::remote::config::RemoteConfig;
use crate::remote::download_source::{SharedDownloadSource, new_shared};
use crate::remote::price_syncer::PriceSyncer;
use crate::crash::logger::{CrashLogger, CrashEntry};
use crate::updater::checker::UpdateInfo;

/// Shared state for remote services
pub struct RemoteState {
    pub price_syncer: PriceSyncer,
    pub config_manager: crate::remote::config::RemoteConfigManager,
    pub crash_logger: Option<CrashLogger>,
    pub github_owner: String,
    pub github_repo: String,
    pub download_source: SharedDownloadSource,
}

impl RemoteState {
    pub fn new(owner: &str, repo: &str, download_source_str: &str) -> Self {
        let download_source = new_shared(download_source_str);
        Self {
            price_syncer: PriceSyncer::new(owner, repo, download_source.clone()),
            config_manager: crate::remote::config::RemoteConfigManager::new(owner, repo, download_source.clone()),
            crash_logger: CrashLogger::new(),
            github_owner: owner.to_string(),
            github_repo: repo.to_string(),
            download_source,
        }
    }
}

pub type RemoteStateManaged = Arc<RemoteState>;

/// Get the current application version
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Check for application updates
#[tauri::command]
pub async fn check_for_update(
    remote: State<'_, RemoteStateManaged>,
) -> Result<Option<UpdateInfo>, String> {
    let checker = crate::updater::checker::UpdateChecker::new(
        env!("CARGO_PKG_VERSION"),
        &remote.github_owner,
        &remote.github_repo,
        remote.download_source.clone(),
    );
    Ok(checker.check_for_update().await)
}

/// Fetch remote app configuration
#[tauri::command]
pub async fn fetch_remote_config(
    remote: State<'_, RemoteStateManaged>,
) -> Result<Option<RemoteConfig>, String> {
    Ok(remote.config_manager.fetch_config().await)
}

/// Force sync remote prices (bypass cache)
#[tauri::command]
pub async fn sync_remote_prices(
    remote: State<'_, RemoteStateManaged>,
) -> Result<u32, String> {
    let prices = remote.price_syncer.force_sync().await;
    let count = prices.len() as u32;

    if count > 0 {
        log::info!("Remote prices synced: {} models", count);
    }

    Ok(count)
}

/// Get crash log entries
#[tauri::command]
pub fn get_crash_logs(
    remote: State<'_, RemoteStateManaged>,
) -> Result<Vec<CrashEntry>, String> {
    match &remote.crash_logger {
        Some(logger) => Ok(logger.list_entries()),
        None => Ok(vec![]),
    }
}

/// Delete a specific crash log entry
#[tauri::command]
pub fn delete_crash_log(
    remote: State<'_, RemoteStateManaged>,
    id: String,
) -> Result<bool, String> {
    match &remote.crash_logger {
        Some(logger) => Ok(logger.delete_entry(&id)),
        None => Ok(false),
    }
}

/// Clear all crash logs
#[tauri::command]
pub fn clear_crash_logs(
    remote: State<'_, RemoteStateManaged>,
) -> Result<u32, String> {
    match &remote.crash_logger {
        Some(logger) => Ok(logger.clear_all() as u32),
        None => Ok(0),
    }
}

/// Generate GitHub Issue URL for a crash entry
#[tauri::command]
pub fn get_crash_issue_url(
    remote: State<'_, RemoteStateManaged>,
    id: String,
) -> Result<String, String> {
    match &remote.crash_logger {
        Some(logger) => {
            let entries = logger.list_entries();
            let prefix: String = id.chars().take(8).collect();
            match entries.iter().find(|e| e.id.starts_with(&prefix)) {
                Some(entry) => Ok(logger.generate_issue_url(
                    entry,
                    &remote.github_owner,
                    &remote.github_repo,
                )),
                None => Err("Crash entry not found".to_string()),
            }
        }
        None => Err("Crash logger not available".to_string()),
    }
}
