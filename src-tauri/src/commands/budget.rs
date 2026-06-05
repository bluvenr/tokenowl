use std::sync::Arc;
use tauri::State;
use tauri_plugin_notification::NotificationExt;
use crate::storage::database::Database;
use crate::storage::queries;
use crate::models::budget::*;

#[tauri::command]
pub async fn get_budget_config(
    db: State<'_, Arc<Database>>,
) -> Result<BudgetConfig, String> {
    queries::get_budget_config(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_budget_config(
    db: State<'_, Arc<Database>>,
    config: BudgetConfig,
) -> Result<(), String> {
    queries::update_budget_config(&db, &config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_budget_alert(
    db: State<'_, Arc<Database>>,
) -> Result<Option<BudgetAlert>, String> {
    let config = queries::get_budget_config(&db).map_err(|e| e.to_string())?;

    // Check daily budget
    if let Some(daily_limit) = config.daily_limit_usd {
        let summary = queries::get_usage_summary(&db, "today").map_err(|e| e.to_string())?;
        let percentage = if daily_limit > 0.0 {
            summary.total_cost_usd / daily_limit * 100.0
        } else {
            0.0
        };

        if percentage >= config.alert_threshold_pct as f64 {
            return Ok(Some(BudgetAlert {
                triggered: true,
                period: "daily".to_string(),
                current_cost: summary.total_cost_usd,
                limit: daily_limit,
                percentage,
                message: format!(
                    "Daily budget alert: ${:.2} / ${:.2} ({:.0}%)",
                    summary.total_cost_usd, daily_limit, percentage
                ),
            }));
        }
    }

    // Check weekly budget
    if let Some(weekly_limit) = config.weekly_limit_usd {
        let summary = queries::get_usage_summary(&db, "week").map_err(|e| e.to_string())?;
        let percentage = if weekly_limit > 0.0 {
            summary.total_cost_usd / weekly_limit * 100.0
        } else {
            0.0
        };

        if percentage >= config.alert_threshold_pct as f64 {
            return Ok(Some(BudgetAlert {
                triggered: true,
                period: "weekly".to_string(),
                current_cost: summary.total_cost_usd,
                limit: weekly_limit,
                percentage,
                message: format!(
                    "Weekly budget alert: ${:.2} / ${:.2} ({:.0}%)",
                    summary.total_cost_usd, weekly_limit, percentage
                ),
            }));
        }
    }

    // Check monthly budget
    if let Some(monthly_limit) = config.monthly_limit_usd {
        let summary = queries::get_usage_summary(&db, "month").map_err(|e| e.to_string())?;
        let percentage = if monthly_limit > 0.0 {
            summary.total_cost_usd / monthly_limit * 100.0
        } else {
            0.0
        };

        if percentage >= config.alert_threshold_pct as f64 {
            return Ok(Some(BudgetAlert {
                triggered: true,
                period: "monthly".to_string(),
                current_cost: summary.total_cost_usd,
                limit: monthly_limit,
                percentage,
                message: format!(
                    "Monthly budget alert: ${:.2} / ${:.2} ({:.0}%)",
                    summary.total_cost_usd, monthly_limit, percentage
                ),
            }));
        }
    }

    Ok(None)
}

#[tauri::command]
pub async fn send_notification(
    app: tauri::AppHandle,
    title: String,
    body: String,
) -> Result<(), String> {
    app.notification()
        .builder()
        .title(&title)
        .body(&body)
        .show()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_db_stats(
    db: State<'_, Arc<Database>>,
) -> Result<DbStats, String> {
    queries::get_db_stats(&db).map_err(|e| e.to_string())
}
