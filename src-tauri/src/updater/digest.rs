//! Daily/Weekly cost digest notification scheduler.
//!
//! Runs a background thread that checks every 60 seconds whether it's time
//! to send a daily or weekly cost summary notification.

use std::sync::Arc;
use chrono::Local;
use tauri_plugin_notification::NotificationExt;
use crate::storage::database::Database;
use crate::storage::queries;

/// Handle for the digest scheduler task.
pub struct DigestHandle {
    pub thread_handle: std::thread::JoinHandle<()>,
}

impl DigestHandle {
    pub fn stop(&self) {
        // Note: std::thread doesn't have abort, thread will stop when process exits
    }
}

impl Drop for DigestHandle {
    fn drop(&mut self) {
        // Thread will be cleaned up when process exits
    }
}

/// Start the digest notification scheduler.
pub fn start_digest_scheduler(
    db: Arc<Database>,
    app_handle: tauri::AppHandle,
) -> DigestHandle {
    log::info!("Starting digest notification scheduler");

    let handle = std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            digest_loop(db, app_handle);
        }));
        if let Err(e) = result {
            log::error!("Digest scheduler thread panicked: {:?}", e);
        }
    });

    DigestHandle {
        thread_handle: handle,
    }
}

fn digest_loop(db: Arc<Database>, app_handle: tauri::AppHandle) {
    let mut last_daily_sent: Option<String> = None;
    let mut last_weekly_sent: Option<String> = None;

    // Skip first check
    std::thread::sleep(std::time::Duration::from_secs(60));

    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));

        let now = Local::now();
        let today = now.format("%Y-%m-%d").to_string();
        let current_time = now.format("%H:%M").to_string();
        let weekday = now.format("%A").to_string();

        // Read settings
        let settings = match queries::get_app_settings(&db) {
            Ok(s) => s,
            Err(e) => {
                log::debug!("Digest: failed to read settings: {}", e);
                continue;
            }
        };

        // Daily digest
        if settings.daily_digest_enabled
            && last_daily_sent.as_deref() != Some(&today)
            && current_time == settings.daily_digest_time
        {
            let body = build_daily_digest(&db);
            let _ = app_handle
                .notification()
                .builder()
                .title("TokenOwl - 每日费用摘要")
                .body(&body)
                .show();
            log::info!("Daily digest sent: {}", body);
            last_daily_sent = Some(today.clone());
        }

        // Weekly digest (every Monday at 09:00)
        if settings.weekly_digest_enabled
            && last_weekly_sent.as_deref() != Some(&today)
            && weekday == "Monday"
            && current_time == "09:00"
        {
            let body = build_weekly_digest(&db);
            let _ = app_handle
                .notification()
                .builder()
                .title("TokenOwl - 每周费用摘要")
                .body(&body)
                .show();
            log::info!("Weekly digest sent: {}", body);
            last_weekly_sent = Some(today.clone());
        }
    }
}

fn build_daily_digest(db: &Arc<Database>) -> String {
    match queries::get_usage_summary(db, "today") {
        Ok(summary) => {
            let cost = format_usd(summary.total_cost_usd);
            let tokens = format_tokens(summary.total_tokens);
            let requests = summary.request_count;

            let mut msg = format!("今日花费: {} | Token: {} | 请求: {}", cost, tokens, requests);

            // Add provider breakdown
            if let Ok(providers) = queries::get_usage_by_provider(db, "today") {
                let top: Vec<_> = providers.iter().take(3).collect();
                if !top.is_empty() {
                    msg.push_str(" | ");
                    let parts: Vec<String> = top
                        .iter()
                        .map(|p| format!("{}: {}", p.provider_name, format_usd(p.cost_usd)))
                        .collect();
                    msg.push_str(&parts.join(", "));
                }
            }

            msg
        }
        Err(e) => {
            log::warn!("Failed to build daily digest: {}", e);
            "获取今日费用数据失败".to_string()
        }
    }
}

fn build_weekly_digest(db: &Arc<Database>) -> String {
    match queries::get_usage_summary(db, "week") {
        Ok(summary) => {
            let cost = format_usd(summary.total_cost_usd);
            let tokens = format_tokens(summary.total_tokens);
            let requests = summary.request_count;

            let mut msg = format!("本周花费: {} | Token: {} | 请求: {}", cost, tokens, requests);

            // Add comparison with last week
            if let Ok(comparison) = queries::get_period_comparison(db, "week") {
                if let Some(change) = comparison.cost_change_pct {
                    let arrow = if change > 0.0 { "↑" } else if change < 0.0 { "↓" } else { "→" };
                    msg.push_str(&format!(" | 环比 {}{:.1}%", arrow, change.abs()));
                }
            }

            msg
        }
        Err(e) => {
            log::warn!("Failed to build weekly digest: {}", e);
            "获取本周费用数据失败".to_string()
        }
    }
}

fn format_usd(amount: f64) -> String {
    if amount >= 1000.0 {
        format!("${:.2}k", amount / 1000.0)
    } else {
        format!("${:.4}", amount)
    }
}

fn format_tokens(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}k", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}
