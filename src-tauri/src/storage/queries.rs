use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use crate::error::AppResult;
use crate::storage::database::Database;
use crate::models::usage::*;
use crate::models::budget::*;
use crate::models::settings::{AppSettings, CustomPrice};

/// Helper to compute period time range
fn period_range(period: &str) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let now = Utc::now();
    match period {
        "today" => {
            let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
            let start = DateTime::<Utc>::from_naive_utc_and_offset(start, Utc);
            Some((start, now))
        }
        "week" => {
            let start = now - Duration::days(7);
            Some((start, now))
        }
        "month" => {
            let start = now - Duration::days(30);
            Some((start, now))
        }
        "all" => None, // No time filter
        _ => {
            let start = now - Duration::days(7);
            Some((start, now))
        }
    }
}

/// Helper to build time filter clause
fn time_filter(period: &str, column: &str) -> (String, Vec<String>) {
    match period_range(period) {
        Some((start, end)) => (
            format!("{} >= ? AND {} <= ?", column, column),
            vec![start.to_rfc3339(), end.to_rfc3339()],
        ),
        None => (String::from("1=1"), vec![]),
    }
}

/// Get usage summary for a period
pub fn get_usage_summary(db: &Arc<Database>, period: &str) -> AppResult<UsageSummary> {
    let conn = db.conn()?;
    let (filter, params) = time_filter(period, "timestamp");

    let query = format!(
        "SELECT
            COALESCE(SUM(cost_usd), 0.0),
            COALESCE(SUM(total_tokens), 0),
            COALESCE(SUM(input_tokens), 0),
            COALESCE(SUM(output_tokens), 0),
            COALESCE(SUM(cache_read_tokens), 0),
            COUNT(DISTINCT cc_switch_log_id),
            COUNT(*)
        FROM usage_records
        WHERE {}",
        filter
    );

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();

    let result = conn.query_row(
        &query,
        params_refs.as_slice(),
        |row| {
            Ok(UsageSummary {
                total_cost_usd: row.get(0)?,
                total_tokens: row.get::<_, u64>(1)?,
                input_tokens: row.get::<_, u64>(2)?,
                output_tokens: row.get::<_, u64>(3)?,
                cache_tokens: row.get::<_, u64>(4)?,
                session_count: row.get::<_, u64>(5)?,
                request_count: row.get::<_, u64>(6)?,
            })
        },
    )?;

    Ok(result)
}

/// Compute previous period time range (shifted back by the same period length)
fn prev_period_range(period: &str) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let now = Utc::now();
    match period {
        "today" => {
            let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
            let today_start = DateTime::<Utc>::from_naive_utc_and_offset(today_start, Utc);
            let yesterday_start = today_start - Duration::days(1);
            Some((yesterday_start, today_start))
        }
        "week" => {
            let cur_start = now - Duration::days(7);
            let prev_start = cur_start - Duration::days(7);
            Some((prev_start, cur_start))
        }
        "month" => {
            let cur_start = now - Duration::days(30);
            let prev_start = cur_start - Duration::days(30);
            Some((prev_start, cur_start))
        }
        _ => None,
    }
}

/// Get period-over-period comparison
pub fn get_period_comparison(db: &Arc<Database>, period: &str) -> AppResult<PeriodComparison> {
    let prev_range = prev_period_range(period);

    // If no previous period (e.g. "all"), return None for all comparisons
    let (prev_start, prev_end) = match prev_range {
        Some(r) => r,
        None => {
            return Ok(PeriodComparison {
                cost_change_pct: None,
                tokens_change_pct: None,
                requests_change_pct: None,
                sessions_change_pct: None,
            });
        }
    };

    let conn = db.conn()?;

    // Query previous period summary
    let (prev_cost, prev_tokens, prev_requests, prev_sessions) = conn.query_row(
        "SELECT
            COALESCE(SUM(cost_usd), 0.0),
            COALESCE(SUM(total_tokens), 0),
            COUNT(*),
            COUNT(DISTINCT cc_switch_log_id)
        FROM usage_records
        WHERE timestamp >= ? AND timestamp < ?",
        rusqlite::params![prev_start.to_rfc3339(), prev_end.to_rfc3339()],
        |row| {
            Ok((
                row.get::<_, f64>(0)?,
                row.get::<_, u64>(1)?,
                row.get::<_, u64>(2)?,
                row.get::<_, u64>(3)?,
            ))
        },
    )?;

    // Query current period summary
    let (filter, params) = time_filter(period, "timestamp");
    let (cur_cost, cur_tokens, cur_requests, cur_sessions) = conn.query_row(
        &format!(
            "SELECT
                COALESCE(SUM(cost_usd), 0.0),
                COALESCE(SUM(total_tokens), 0),
                COUNT(*),
                COUNT(DISTINCT cc_switch_log_id)
            FROM usage_records
            WHERE {}",
            filter
        ),
        rusqlite::params_from_iter(params.iter()),
        |row| {
            Ok((
                row.get::<_, f64>(0)?,
                row.get::<_, u64>(1)?,
                row.get::<_, u64>(2)?,
                row.get::<_, u64>(3)?,
            ))
        },
    )?;

    let pct_change = |cur: f64, prev: f64| -> Option<f64> {
        if prev == 0.0 {
            if cur == 0.0 { Some(0.0) } else { None }
        } else {
            Some(((cur - prev) / prev) * 100.0)
        }
    };

    Ok(PeriodComparison {
        cost_change_pct: pct_change(cur_cost, prev_cost),
        tokens_change_pct: pct_change(cur_tokens as f64, prev_tokens as f64),
        requests_change_pct: pct_change(cur_requests as f64, prev_requests as f64),
        sessions_change_pct: pct_change(cur_sessions as f64, prev_sessions as f64),
    })
}

/// Get usage breakdown by model
pub fn get_usage_by_model(db: &Arc<Database>, period: &str) -> AppResult<Vec<ModelUsage>> {
    let conn = db.conn()?;
    let (filter, params) = time_filter(period, "timestamp");

    // First get total cost for percentage calculation
    let total_cost: f64 = conn.query_row(
        &format!("SELECT COALESCE(SUM(cost_usd), 0.0) FROM usage_records WHERE {}", filter),
        rusqlite::params_from_iter(params.iter()),
        |row| row.get(0),
    )?;

    let query = format!(
        "SELECT model,
            COALESCE(SUM(cost_usd), 0.0) as cost,
            COALESCE(SUM(total_tokens), 0),
            COUNT(*)
        FROM usage_records
        WHERE {}
        GROUP BY model
        ORDER BY cost DESC",
        filter
    );

    let mut stmt = conn.prepare(&query)?;
    let entries: Vec<ModelUsage> = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |row| {
            let cost: f64 = row.get(1)?;
            Ok(ModelUsage {
                model: row.get(0)?,
                cost_usd: cost,
                total_tokens: row.get(2)?,
                request_count: row.get(3)?,
                percentage: if total_cost > 0.0 { cost / total_cost * 100.0 } else { 0.0 },
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(entries)
}

/// Get usage breakdown by provider
pub fn get_usage_by_provider(db: &Arc<Database>, period: &str) -> AppResult<Vec<ProviderUsage>> {
    let conn = db.conn()?;
    let (filter, params) = time_filter(period, "timestamp");

    let total_cost: f64 = conn.query_row(
        &format!("SELECT COALESCE(SUM(cost_usd), 0.0) FROM usage_records WHERE {}", filter),
        rusqlite::params_from_iter(params.iter()),
        |row| row.get(0),
    )?;

    let query = format!(
        "SELECT
            COALESCE(provider_name, 'unknown') as provider,
            COALESCE(SUM(cost_usd), 0.0) as cost,
            COALESCE(SUM(total_tokens), 0),
            COUNT(*) as req_count,
            COALESCE(AVG(response_time_ms), 0.0),
            COALESCE(SUM(CASE WHEN status_code >= 400 THEN 1 ELSE 0 END) * 1.0 / COUNT(*), 0.0)
        FROM usage_records
        WHERE {}
        GROUP BY provider
        ORDER BY cost DESC",
        filter
    );

    let mut stmt = conn.prepare(&query)?;
    let entries: Vec<ProviderUsage> = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |row| {
            let cost: f64 = row.get(1)?;
            Ok(ProviderUsage {
                provider_name: row.get(0)?,
                cost_usd: cost,
                total_tokens: row.get(2)?,
                request_count: row.get(3)?,
                avg_latency_ms: row.get(4)?,
                failure_rate: row.get(5)?,
                percentage: if total_cost > 0.0 { cost / total_cost * 100.0 } else { 0.0 },
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(entries)
}

/// Get cost attribution tree: Provider → Model → Token type
pub fn get_cost_attribution(db: &Arc<Database>, period: &str) -> AppResult<Vec<ProviderAttribution>> {
    let conn = db.conn()?;
    let (filter, params) = time_filter(period, "timestamp");

    // Get total cost across all providers
    let total_cost: f64 = conn.query_row(
        &format!("SELECT COALESCE(SUM(cost_usd), 0.0) FROM usage_records WHERE {}", filter),
        rusqlite::params_from_iter(params.iter()),
        |row| row.get(0),
    )?;

    // Get providers with costs, sorted by cost descending
    let provider_query = format!(
        "SELECT COALESCE(provider_name, 'unknown') as provider,
            COALESCE(SUM(cost_usd), 0.0) as cost,
            COALESCE(SUM(total_tokens), 0)
        FROM usage_records
        WHERE {}
        GROUP BY provider
        ORDER BY cost DESC",
        filter
    );

    let mut provider_stmt = conn.prepare(&provider_query)?;
    let providers: Vec<(String, f64, u64)> = provider_stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut result: Vec<ProviderAttribution> = Vec::new();

    for (provider_name, provider_cost, _) in providers {
        let provider_pct = if total_cost > 0.0 {
            provider_cost / total_cost * 100.0
        } else {
            0.0
        };

        // Get models for this provider
        let model_query = format!(
            "SELECT model,
                COALESCE(SUM(cost_usd), 0.0) as cost,
                COALESCE(SUM(total_tokens), 0),
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(cache_creation_tokens), 0),
                COALESCE(SUM(cache_read_tokens), 0),
                COALESCE(SUM(reasoning_tokens), 0)
            FROM usage_records
            WHERE {} AND COALESCE(provider_name, 'unknown') = ?
            GROUP BY model
            ORDER BY cost DESC",
            filter
        );

        let mut model_stmt = conn.prepare(&model_query)?;
        let model_params: Vec<String> = params.iter().cloned().chain(std::iter::once(provider_name.clone())).collect();
        let models: Vec<(String, f64, u64, u64, u64, u64, u64, u64)> = model_stmt
            .query_map(rusqlite::params_from_iter(model_params.iter()), |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let mut model_attributions: Vec<ModelAttribution> = Vec::new();

        for (model, model_cost, model_tokens, input, output, cache_write, cache_read, reasoning) in models {
            let model_pct = if provider_cost > 0.0 {
                model_cost / provider_cost * 100.0
            } else {
                0.0
            };

            // Build token breakdown
            let mut token_breakdown: Vec<TokenBreakdown> = Vec::new();
            let token_types = [
                ("input", input),
                ("output", output),
                ("cache_write", cache_write),
                ("cache_read", cache_read),
                ("reasoning", reasoning),
            ];

            for (ttype, count) in token_types {
                if count > 0 {
                    let pct = if model_tokens > 0 {
                        count as f64 / model_tokens as f64 * 100.0
                    } else {
                        0.0
                    };
                    // Approximate cost allocation by token proportion
                    let cost_share = if model_tokens > 0 {
                        model_cost * (count as f64 / model_tokens as f64)
                    } else {
                        0.0
                    };
                    token_breakdown.push(TokenBreakdown {
                        token_type: ttype.to_string(),
                        cost_usd: cost_share,
                        tokens: count,
                        percentage: pct,
                    });
                }
            }

            // Sort by tokens descending
            token_breakdown.sort_by(|a, b| b.tokens.cmp(&a.tokens));

            model_attributions.push(ModelAttribution {
                model,
                cost_usd: model_cost,
                total_tokens: model_tokens,
                token_breakdown,
                percentage: model_pct,
            });
        }

        result.push(ProviderAttribution {
            provider_name,
            cost_usd: provider_cost,
            models: model_attributions,
            percentage: provider_pct,
        });
    }

    Ok(result)
}

/// Get budget burn rate analysis
pub fn get_budget_burn_rate(db: &Arc<Database>) -> AppResult<BudgetBurnRate> {
    let conn = db.conn()?;

    // Get budget config
    let budget = conn.query_row(
        "SELECT daily_limit_usd, weekly_limit_usd, monthly_limit_usd FROM budget_config WHERE id = 1",
        [],
        |row| {
            Ok((
                row.get::<_, Option<f64>>(0)?,
                row.get::<_, Option<f64>>(1)?,
                row.get::<_, Option<f64>>(2)?,
            ))
        },
    )?;

    let now = Utc::now();

    // Calculate daily rate (average daily cost over last 7 days)
    let week_ago = now - Duration::days(7);
    let week_cost: f64 = conn.query_row(
        "SELECT COALESCE(SUM(cost_usd), 0.0) FROM usage_records WHERE timestamp >= ?",
        rusqlite::params![week_ago.to_rfc3339()],
        |row| row.get(0),
    )?;
    let daily_rate = if week_cost > 0.0 { week_cost / 7.0 } else { 0.0 };

    // Today's spend
    let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let today_start = DateTime::<Utc>::from_naive_utc_and_offset(today_start, Utc);
    let daily_spend: f64 = conn.query_row(
        "SELECT COALESCE(SUM(cost_usd), 0.0) FROM usage_records WHERE timestamp >= ?",
        rusqlite::params![today_start.to_rfc3339()],
        |row| row.get(0),
    )?;

    // Weekly spend (last 7 days)
    let weekly_spend: f64 = conn.query_row(
        "SELECT COALESCE(SUM(cost_usd), 0.0) FROM usage_records WHERE timestamp >= ?",
        rusqlite::params![week_ago.to_rfc3339()],
        |row| row.get(0),
    )?;

    // Monthly spend (last 30 days)
    let month_ago = now - Duration::days(30);
    let monthly_spend: f64 = conn.query_row(
        "SELECT COALESCE(SUM(cost_usd), 0.0) FROM usage_records WHERE timestamp >= ?",
        rusqlite::params![month_ago.to_rfc3339()],
        |row| row.get(0),
    )?;

    // Calculate days remaining for each budget
    let calc_days_remaining = |current_spend: f64, limit: Option<f64>, rate: f64| -> Option<f64> {
        match limit {
            Some(l) if l > 0.0 && rate > 0.0 => {
                let remaining = l - current_spend;
                if remaining > 0.0 {
                    Some(remaining / rate)
                } else {
                    Some(0.0)
                }
            }
            _ => None,
        }
    };

    Ok(BudgetBurnRate {
        daily_rate,
        daily_spend: Some(daily_spend),
        daily_limit: budget.0,
        daily_days_remaining: calc_days_remaining(daily_spend, budget.0, daily_rate),
        weekly_spend: Some(weekly_spend),
        weekly_limit: budget.1,
        weekly_days_remaining: calc_days_remaining(weekly_spend, budget.1, daily_rate),
        monthly_spend: Some(monthly_spend),
        monthly_limit: budget.2,
        monthly_days_remaining: calc_days_remaining(monthly_spend, budget.2, daily_rate),
    })
}

/// Get usage trend data points
pub fn get_usage_trend(db: &Arc<Database>, granularity: &str, period: &str) -> AppResult<Vec<TrendPoint>> {
    let conn = db.conn()?;
    let (filter, params) = time_filter(period, "timestamp");

    let group_by = match granularity {
        "hourly" => "strftime('%Y-%m-%d %H:00', timestamp)",
        "daily" => "strftime('%Y-%m-%d', timestamp)",
        "weekly" => "strftime('%Y-W%W', timestamp)",
        _ => "strftime('%Y-%m-%d', timestamp)",
    };

    let query = format!(
        "SELECT {group_by} as ts,
            COALESCE(SUM(cost_usd), 0.0),
            COALESCE(SUM(total_tokens), 0)
        FROM usage_records
        WHERE {}
        GROUP BY ts
        ORDER BY ts ASC",
        filter
    );

    let mut stmt = conn.prepare(&query)?;
    let entries: Vec<TrendPoint> = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(TrendPoint {
                timestamp: row.get(0)?,
                cost_usd: row.get(1)?,
                total_tokens: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(entries)
}

/// Get cache hit rate trend over time
pub fn get_cache_trend(db: &Arc<Database>, granularity: &str, period: &str) -> AppResult<Vec<CacheTrendPoint>> {
    let conn = db.conn()?;
    let (filter, params) = time_filter(period, "timestamp");

    let group_by = match granularity {
        "hourly" => "strftime('%Y-%m-%d %H:00', timestamp)",
        "daily" => "strftime('%Y-%m-%d', timestamp)",
        "weekly" => "strftime('%Y-W%W', timestamp)",
        _ => "strftime('%Y-%m-%d', timestamp)",
    };

    let query = format!(
        "SELECT {group_by} as ts,
            COALESCE(SUM(cache_read_tokens) * 1.0 / NULLIF(SUM(total_tokens), 0), 0.0) as hit_rate,
            COALESCE(SUM(cache_read_tokens), 0),
            COALESCE(SUM(total_tokens), 0)
        FROM usage_records
        WHERE {}
        GROUP BY ts
        ORDER BY ts ASC",
        filter
    );

    let mut stmt = conn.prepare(&query)?;
    let entries: Vec<CacheTrendPoint> = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(CacheTrendPoint {
                timestamp: row.get(0)?,
                cache_hit_rate: row.get::<_, f64>(1)? * 100.0,
                cache_tokens: row.get(2)?,
                total_tokens: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(entries)
}

/// Get recent sessions/requests
pub fn get_recent_sessions(db: &Arc<Database>, limit: u32) -> AppResult<Vec<SessionSummary>> {
    let conn = db.conn()?;

    let mut stmt = conn.prepare(
        "SELECT id, timestamp, model, provider_name, input_tokens, output_tokens, total_tokens, cost_usd, status_code, response_time_ms
        FROM usage_records
        ORDER BY timestamp DESC
        LIMIT ?1"
    )?;

    let entries: Vec<SessionSummary> = stmt
        .query_map([limit], |row| {
            let ts_str: String = row.get(1)?;
            let timestamp = DateTime::parse_from_rfc3339(&ts_str)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc);

            Ok(SessionSummary {
                id: row.get(0)?,
                timestamp,
                model: row.get(2)?,
                provider_name: row.get(3)?,
                input_tokens: row.get(4)?,
                output_tokens: row.get(5)?,
                total_tokens: row.get(6)?,
                cost_usd: row.get(7)?,
                status_code: row.get(8)?,
                response_time_ms: row.get(9)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(entries)
}

/// Get budget config
pub fn get_budget_config(db: &Arc<Database>) -> AppResult<BudgetConfig> {
    let conn = db.conn()?;
    let result = conn.query_row(
        "SELECT daily_limit_usd, weekly_limit_usd, monthly_limit_usd,
            alert_threshold_pct, alert_icon_color, alert_system_notify, alert_dashboard_banner
        FROM budget_config WHERE id = 1",
        [],
        |row| {
            Ok(BudgetConfig {
                daily_limit_usd: row.get(0)?,
                weekly_limit_usd: row.get(1)?,
                monthly_limit_usd: row.get(2)?,
                alert_threshold_pct: row.get(3)?,
                alert_icon_color: row.get(4)?,
                alert_system_notify: row.get(5)?,
                alert_dashboard_banner: row.get(6)?,
            })
        },
    )?;
    Ok(result)
}

/// Update budget config
pub fn update_budget_config(db: &Arc<Database>, config: &BudgetConfig) -> AppResult<()> {
    let conn = db.conn()?;
    conn.execute(
        "UPDATE budget_config SET
            daily_limit_usd = ?1,
            weekly_limit_usd = ?2,
            monthly_limit_usd = ?3,
            alert_threshold_pct = ?4,
            alert_icon_color = ?5,
            alert_system_notify = ?6,
            alert_dashboard_banner = ?7
        WHERE id = 1",
        rusqlite::params![
            config.daily_limit_usd,
            config.weekly_limit_usd,
            config.monthly_limit_usd,
            config.alert_threshold_pct,
            config.alert_icon_color,
            config.alert_system_notify,
            config.alert_dashboard_banner,
        ],
    )?;
    Ok(())
}

/// Get app settings
pub fn get_app_settings(db: &Arc<Database>) -> AppResult<AppSettings> {
    let conn = db.conn()?;
    let result = conn.query_row(
        "SELECT language, download_source, auto_start, theme, tray_display,
            telemetry_enabled, crash_log_enabled, anomaly_threshold, forecast_method,
            data_retention_days, daily_digest_enabled, daily_digest_time,
            weekly_digest_enabled, update_check_interval_hours, price_sync_interval_hours,
            default_period
        FROM app_settings WHERE id = 1",
        [],
        |row| {
            Ok(AppSettings {
                language: row.get(0)?,
                download_source: row.get(1)?,
                auto_start: row.get(2)?,
                theme: row.get(3)?,
                tray_display: row.get(4)?,
                telemetry_enabled: row.get(5)?,
                crash_log_enabled: row.get(6)?,
                anomaly_threshold: row.get(7)?,
                forecast_method: row.get(8)?,
                data_retention_days: row.get(9)?,
                daily_digest_enabled: row.get(10)?,
                daily_digest_time: row.get(11)?,
                weekly_digest_enabled: row.get(12)?,
                update_check_interval_hours: row.get(13)?,
                price_sync_interval_hours: row.get(14)?,
                default_period: row.get(15)?,
            })
        },
    )?;
    Ok(result)
}

/// Update app settings
pub fn update_app_settings(db: &Arc<Database>, settings: &AppSettings) -> AppResult<()> {
    let conn = db.conn()?;
    conn.execute(
        "UPDATE app_settings SET
            language = ?1, download_source = ?2, auto_start = ?3, theme = ?4,
            tray_display = ?5, telemetry_enabled = ?6, crash_log_enabled = ?7,
            anomaly_threshold = ?8, forecast_method = ?9, data_retention_days = ?10,
            daily_digest_enabled = ?11, daily_digest_time = ?12,
            weekly_digest_enabled = ?13, update_check_interval_hours = ?14,
            price_sync_interval_hours = ?15, default_period = ?16
        WHERE id = 1",
        rusqlite::params![
            settings.language, settings.download_source, settings.auto_start,
            settings.theme, settings.tray_display, settings.telemetry_enabled,
            settings.crash_log_enabled, settings.anomaly_threshold, settings.forecast_method,
            settings.data_retention_days, settings.daily_digest_enabled,
            settings.daily_digest_time, settings.weekly_digest_enabled,
            settings.update_check_interval_hours, settings.price_sync_interval_hours,
            settings.default_period,
        ],
    )?;
    Ok(())
}

/// Get database statistics
pub fn get_db_stats(db: &Arc<Database>) -> AppResult<DbStats> {
    let conn = db.conn()?;

    let total_records: u64 = conn.query_row("SELECT COUNT(*) FROM usage_records", [], |row| row.get(0))?;
    let total_cost: f64 = conn.query_row("SELECT COALESCE(SUM(cost_usd), 0.0) FROM usage_records", [], |row| row.get(0))?;
    let min_ts: Option<String> = conn.query_row("SELECT MIN(timestamp) FROM usage_records", [], |row| row.get(0))?;
    let max_ts: Option<String> = conn.query_row("SELECT MAX(timestamp) FROM usage_records", [], |row| row.get(0))?;

    // Get database file size
    let db_size = conn.query_row("SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()", [], |row| {
        row.get::<_, u64>(0)
    }).unwrap_or(0);

    Ok(DbStats {
        total_records,
        total_cost_usd: total_cost,
        date_range_start: min_ts,
        date_range_end: max_ts,
        db_size_bytes: db_size,
    })
}

/// Get custom prices from database
pub fn get_custom_prices(db: &Arc<Database>) -> AppResult<Vec<CustomPrice>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT model_id, input_per_million, output_per_million, cache_write_per_million, cache_read_per_million
        FROM custom_prices"
    )?;

    let entries: Vec<CustomPrice> = stmt
        .query_map([], |row| {
            Ok(CustomPrice {
                model_id: row.get(0)?,
                input_per_million: row.get(1)?,
                output_per_million: row.get(2)?,
                cache_write_per_million: row.get(3)?,
                cache_read_per_million: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(entries)
}

/// Upsert custom price
pub fn upsert_custom_price(db: &Arc<Database>, price: &CustomPrice) -> AppResult<()> {
    let conn = db.conn()?;
    conn.execute(
        "INSERT OR REPLACE INTO custom_prices (model_id, input_per_million, output_per_million, cache_write_per_million, cache_read_per_million)
        VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            price.model_id, price.input_per_million, price.output_per_million,
            price.cache_write_per_million, price.cache_read_per_million,
        ],
    )?;
    Ok(())
}

/// Delete custom price
pub fn delete_custom_price(db: &Arc<Database>, model_id: &str) -> AppResult<()> {
    let conn = db.conn()?;
    conn.execute("DELETE FROM custom_prices WHERE model_id = ?1", [model_id])?;
    Ok(())
}

/// Get models without prices
pub fn get_models_without_prices(db: &Arc<Database>) -> AppResult<Vec<String>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT DISTINCT model FROM usage_records WHERE cost_usd IS NULL"
    )?;

    let models: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(models)
}

/// Count records per model
pub fn count_model_records(db: &Arc<Database>) -> AppResult<Vec<(String, u64)>> {
    let conn = db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT model, COUNT(*) FROM usage_records GROUP BY model ORDER BY COUNT(*) DESC"
    )?;

    let entries: Vec<(String, u64)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(entries)
}

/// Get cost anomaly report
pub fn get_cost_anomalies(db: &Arc<Database>, period: &str) -> AppResult<CostAnomalyReport> {
    let conn = db.conn()?;
    let (filter, params) = time_filter(period, "timestamp");

    // Read anomaly threshold from settings
    let threshold: f64 = conn.query_row(
        "SELECT anomaly_threshold FROM app_settings WHERE id = 1",
        [],
        |row| row.get(0),
    ).unwrap_or(2.5);

    // Get daily costs
    let daily_query = format!(
        "SELECT DATE(timestamp) as day,
            COALESCE(SUM(cost_usd), 0.0) as daily_cost
        FROM usage_records
        WHERE {}
        GROUP BY day
        ORDER BY day ASC",
        filter
    );
    let mut stmt = conn.prepare(&daily_query)?;
    let days: Vec<(String, f64)> = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let total_days = days.len() as u64;

    if days.is_empty() {
        return Ok(CostAnomalyReport {
            anomaly_days: vec![],
            total_days: 0,
            avg_daily_cost: 0.0,
            stddev: 0.0,
            threshold,
        });
    }

    // Compute mean
    let sum: f64 = days.iter().map(|(_, c)| c).sum();
    let mean = sum / days.len() as f64;

    // Compute stddev
    let variance = days.iter().map(|(_, c)| (c - mean).powi(2)).sum::<f64>() / days.len() as f64;
    let stddev = variance.sqrt();

    // If stddev is 0 (all days have same cost), no anomalies possible
    if stddev == 0.0 {
        return Ok(CostAnomalyReport {
            anomaly_days: vec![],
            total_days,
            avg_daily_cost: mean,
            stddev: 0.0,
            threshold,
        });
    }

    let cutoff = mean + threshold * stddev;

    // Find anomalous days
    let mut anomaly_days: Vec<CostAnomaly> = Vec::new();
    for (day, cost) in &days {
        if *cost > cutoff {
            let deviation = (cost - mean) / stddev;

            // Find top provider for this day
            let top_provider: Option<String> = conn.query_row(
                "SELECT provider_name FROM usage_records
                 WHERE DATE(timestamp) = ? AND provider_name IS NOT NULL
                 GROUP BY provider_name ORDER BY SUM(cost_usd) DESC LIMIT 1",
                rusqlite::params![day],
                |row| row.get(0),
            ).ok().flatten();

            // Find top model for this day
            let top_model: Option<String> = conn.query_row(
                "SELECT model FROM usage_records
                 WHERE DATE(timestamp) = ?
                 GROUP BY model ORDER BY SUM(cost_usd) DESC LIMIT 1",
                rusqlite::params![day],
                |row| row.get(0),
            ).ok().flatten();

            anomaly_days.push(CostAnomaly {
                date: day.clone(),
                cost_usd: *cost,
                avg_cost: mean,
                deviation,
                top_provider,
                top_model,
            });
        }
    }

    // Sort by deviation descending
    anomaly_days.sort_by(|a, b| b.deviation.partial_cmp(&a.deviation).unwrap_or(std::cmp::Ordering::Equal));

    Ok(CostAnomalyReport {
        anomaly_days,
        total_days,
        avg_daily_cost: mean,
        stddev,
        threshold,
    })
}

/// Export usage records as CSV string
pub fn export_csv(db: &Arc<Database>, period: &str) -> AppResult<String> {
    let conn = db.conn()?;
    let (filter, params) = time_filter(period, "timestamp");

    let query = format!(
        "SELECT timestamp, app_type, provider_name, model,
            input_tokens, output_tokens, cache_creation_tokens, cache_read_tokens,
            reasoning_tokens, total_tokens, cost_usd, status_code, response_time_ms
        FROM usage_records
        WHERE {}
        ORDER BY timestamp ASC",
        filter
    );

    let mut stmt = conn.prepare(&query)?;

    let mut csv = String::from("timestamp,app_type,provider,model,input_tokens,output_tokens,cache_creation_tokens,cache_read_tokens,reasoning_tokens,total_tokens,cost_usd,status_code,response_time_ms\n");

    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            row.get::<_, String>(3)?,
            row.get::<_, u64>(4)?,
            row.get::<_, u64>(5)?,
            row.get::<_, u64>(6)?,
            row.get::<_, u64>(7)?,
            row.get::<_, u64>(8)?,
            row.get::<_, u64>(9)?,
            row.get::<_, Option<f64>>(10)?.map_or("".to_string(), |v| format!("{:.6}", v)),
            row.get::<_, Option<u16>>(11)?.map_or("".to_string(), |v| v.to_string()),
            row.get::<_, Option<u64>>(12)?.map_or("".to_string(), |v| v.to_string()),
        ))
    })?;

    for row in rows {
        if let Ok(line) = row {
            csv.push_str(&line);
            csv.push('\n');
        }
    }

    Ok(csv)
}

/// Export usage records as JSON string
pub fn export_json(db: &Arc<Database>, period: &str) -> AppResult<String> {
    let conn = db.conn()?;
    let (filter, params) = time_filter(period, "timestamp");

    let query = format!(
        "SELECT timestamp, app_type, provider_name, model,
            input_tokens, output_tokens, cache_creation_tokens, cache_read_tokens,
            reasoning_tokens, total_tokens, cost_usd
        FROM usage_records
        WHERE {}
        ORDER BY timestamp ASC",
        filter
    );

    let mut stmt = conn.prepare(&query)?;

    let rows: Vec<serde_json::Value> = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(serde_json::json!({
                "timestamp": row.get::<_, String>(0)?,
                "app_type": row.get::<_, String>(1)?,
                "provider": row.get::<_, Option<String>>(2)?,
                "model": row.get::<_, String>(3)?,
                "input_tokens": row.get::<_, u64>(4)?,
                "output_tokens": row.get::<_, u64>(5)?,
                "cache_creation_tokens": row.get::<_, u64>(6)?,
                "cache_read_tokens": row.get::<_, u64>(7)?,
                "reasoning_tokens": row.get::<_, u64>(8)?,
                "total_tokens": row.get::<_, u64>(9)?,
                "cost_usd": row.get::<_, Option<f64>>(10)?,
            }))
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(serde_json::to_string_pretty(&rows)?)
}
