use chrono::{Datelike, Duration, TimeZone, Utc};
use rusqlite::params;
use std::collections::HashMap;

use crate::error::AppResult;
use crate::models::usage::*;
use crate::models::budget::BudgetConfig;
use crate::models::settings::{AppSettings, ModelPricing};
use super::database::Database;

// ─── Usage Records ───────────────────────────────────────────────────

impl Database {
    /// Insert usage records (skip duplicates by id)
    pub fn insert_records(&self, records: &[UsageRecord]) -> AppResult<usize> {
        let mut conn = self.conn()?;
        let tx = conn.transaction()?;
        let mut count = 0;
        for r in records {
            tx.execute(
                "INSERT OR IGNORE INTO usage_records
                    (id, source, session_id, timestamp, model,
                     input_tokens, output_tokens, cache_creation_tokens,
                     cache_read_tokens, total_tokens, reasoning_tokens,
                     cost_usd, project_path, provider_name,
                     response_time_ms, status_code, cc_switch_log_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                params![
                    r.id,
                    r.source.as_str(),
                    r.session_id,
                    r.timestamp.to_rfc3339(),
                    r.model,
                    r.tokens.input_tokens,
                    r.tokens.output_tokens,
                    r.tokens.cache_creation_tokens,
                    r.tokens.cache_read_tokens,
                    r.tokens.total_tokens,
                    r.tokens.reasoning_tokens,
                    r.cost_usd,
                    r.project_path,
                    r.provider_name,
                    r.response_time_ms.map(|v| v as i64),
                    r.status_code.map(|v| v as i64),
                    r.cc_switch_log_id,
                ],
            )?;
            // Only count if the row was actually inserted (not ignored as duplicate)
            if tx.changes() > 0 {
                count += 1;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// Get usage summary for a period
    pub fn get_usage_summary(&self, period: &str) -> AppResult<UsageSummary> {
        let (start, end) = period_to_range(period)?;
        let conn = self.conn()?;

        let result = conn.query_row(
            "SELECT COALESCE(SUM(cost_usd), 0),
                    COALESCE(SUM(total_tokens), 0),
                    COALESCE(SUM(input_tokens), 0),
                    COALESCE(SUM(output_tokens), 0),
                    COALESCE(SUM(reasoning_tokens), 0),
                    COUNT(DISTINCT session_id)
             FROM usage_records
             WHERE timestamp >= ?1 AND timestamp < ?2",
            params![start, end],
            |row| {
                Ok(UsageSummary {
                    total_cost_usd: row.get(0)?,
                    total_tokens: row.get::<_, i64>(1)? as u64,
                    input_tokens: row.get::<_, i64>(2)? as u64,
                    output_tokens: row.get::<_, i64>(3)? as u64,
                    reasoning_tokens: row.get::<_, i64>(4)? as u64,
                    session_count: row.get::<_, i64>(5)? as u64,
                })
            },
        )?;
        Ok(result)
    }

    /// Get usage breakdown by model
    pub fn get_usage_by_model(&self, period: &str) -> AppResult<Vec<ModelUsage>> {
        let (start, end) = period_to_range(period)?;
        let conn = self.conn()?;

        let mut stmt = conn.prepare(
            "SELECT model, source,
                    COALESCE(SUM(cost_usd), 0),
                    COALESCE(SUM(total_tokens), 0),
                    COALESCE(SUM(input_tokens), 0),
                    COALESCE(SUM(output_tokens), 0),
                    COALESCE(SUM(reasoning_tokens), 0)
             FROM usage_records
             WHERE timestamp >= ?1 AND timestamp < ?2
             GROUP BY model, source
             ORDER BY SUM(cost_usd) DESC",
        )?;

        let result = stmt
            .query_map(params![start, end], |row| {
                Ok(ModelUsage {
                    model: row.get(0)?,
                    source: row.get(1)?,
                    cost_usd: row.get(2)?,
                    total_tokens: row.get::<_, i64>(3)? as u64,
                    input_tokens: row.get::<_, i64>(4)? as u64,
                    output_tokens: row.get::<_, i64>(5)? as u64,
                    reasoning_tokens: row.get::<_, i64>(6)? as u64,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(result)
    }

    /// Get trend data points
    pub fn get_usage_trend(
        &self,
        granularity: &str,
        period: &str,
    ) -> AppResult<Vec<TrendPoint>> {
        let (start, end) = period_to_range(period)?;
        let conn = self.conn()?;

        // Use 'localtime' modifier so strftime buckets timestamps in the user's local timezone
        let date_format = match granularity {
            "hourly" => "%Y-%m-%d %H:00",
            "daily" => "%Y-%m-%d",
            "weekly" => "%Y-W%W",
            _ => "%Y-%m-%d",
        };

        let mut stmt = conn.prepare(&format!(
            "SELECT strftime('{}', timestamp, 'localtime') as date_bucket,
                    COALESCE(SUM(cost_usd), 0),
                    COALESCE(SUM(total_tokens), 0)
             FROM usage_records
             WHERE timestamp >= ?1 AND timestamp < ?2
             GROUP BY date_bucket
             ORDER BY date_bucket ASC",
            date_format
        ))?;

        let result = stmt
            .query_map(params![start, end], |row| {
                Ok(TrendPoint {
                    date: row.get(0)?,
                    cost_usd: row.get(1)?,
                    total_tokens: row.get::<_, i64>(2)? as u64,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(result)
    }

    /// Get recent sessions
    pub fn get_recent_sessions(&self, limit: u32) -> AppResult<Vec<SessionSummary>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT session_id, source, model,
                    COALESCE(SUM(cost_usd), 0),
                    COALESCE(SUM(total_tokens), 0),
                    MAX(timestamp),
                    project_path
             FROM usage_records
             GROUP BY session_id, source
             ORDER BY MAX(timestamp) DESC
             LIMIT ?1",
        )?;

        let result = stmt
            .query_map(params![limit], |row| {
                Ok(SessionSummary {
                    session_id: row.get(0)?,
                    source: row.get(1)?,
                    model: row.get(2)?,
                    cost_usd: row.get(3)?,
                    total_tokens: row.get::<_, i64>(4)? as u64,
                    timestamp: row.get(5)?,
                    project_path: row.get(6)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(result)
    }

    /// Get all records that don't have a cost_usd value (for cost backfill)
    pub fn get_records_without_cost(&self, limit: u32) -> AppResult<Vec<UsageRecord>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, source, session_id, timestamp, model,
                    input_tokens, output_tokens, cache_creation_tokens,
                    cache_read_tokens, total_tokens, reasoning_tokens,
                    project_path, provider_name, response_time_ms,
                    status_code, cc_switch_log_id
             FROM usage_records
             WHERE cost_usd IS NULL
             ORDER BY timestamp DESC
             LIMIT ?1",
        )?;

        let result = stmt
            .query_map(params![limit], |row| {
                let source_str: String = row.get(1)?;
                let ts_str: String = row.get(3)?;
                Ok(UsageRecord {
                    id: row.get(0)?,
                    source: DataSource::from_str(&source_str)
                        .unwrap_or_else(|| {
                            log::warn!("Unknown data source in database: {}", source_str);
                            DataSource::CcSwitch
                        }),
                    session_id: row.get(2)?,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&ts_str)
                        .unwrap_or_else(|_| Utc::now().into())
                        .with_timezone(&Utc),
                    model: row.get(4)?,
                    tokens: TokenUsage {
                        input_tokens: row.get::<_, i64>(5)? as u64,
                        output_tokens: row.get::<_, i64>(6)? as u64,
                        cache_creation_tokens: row.get::<_, i64>(7)? as u64,
                        cache_read_tokens: row.get::<_, i64>(8)? as u64,
                        total_tokens: row.get::<_, i64>(9)? as u64,
                        reasoning_tokens: row.get::<_, i64>(10)? as u64,
                    },
                    cost_usd: None,
                    project_path: row.get(11)?,
                    provider_name: row.get(12)?,
                    response_time_ms: row.get::<_, Option<i64>>(13)?.map(|v| v as u64),
                    status_code: row.get::<_, Option<i64>>(14)?.map(|v| v as u16),
                    cc_switch_log_id: row.get(15)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(result)
    }

    /// Get distinct models that have records but no price configured
    pub fn get_models_without_prices(&self) -> AppResult<Vec<MissingModelPrice>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT DISTINCT model, source FROM usage_records WHERE cost_usd IS NULL ORDER BY model",
        )?;
        let result = stmt
            .query_map([], |row| {
                Ok(MissingModelPrice {
                    model: row.get(0)?,
                    source: row.get(1)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(result)
    }

    /// Update cost_usd for a specific record
    pub fn update_record_cost(&self, id: &str, cost_usd: f64) -> AppResult<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE usage_records SET cost_usd = ?2 WHERE id = ?1",
            params![id, cost_usd],
        )?;
        Ok(())
    }

    // ─── Sync State ─────────────────────────────────────────────────

    pub fn get_sync_state(&self) -> AppResult<SyncState> {
        let conn = self.conn()?;
        let result = conn.query_row(
            "SELECT last_sync_time, last_sync_record_count, cc_switch_db_path,
                    cc_switch_detected, sync_interval_secs, total_records_synced
             FROM sync_state WHERE id = 1",
            [],
            |row| {
                Ok(SyncState {
                    last_sync_time: row.get(0)?,
                    last_sync_record_count: row.get::<_, i64>(1)? as u64,
                    cc_switch_db_path: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    cc_switch_detected: row.get(3)?,
                    sync_interval_secs: row.get::<_, i64>(4)? as u64,
                    total_records_synced: row.get::<_, i64>(5)? as u64,
                })
            },
        )?;
        Ok(result)
    }

    pub fn update_sync_state(&self, state: &SyncState) -> AppResult<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE sync_state SET
                last_sync_time = ?1, last_sync_record_count = ?2,
                cc_switch_db_path = ?3, cc_switch_detected = ?4,
                sync_interval_secs = ?5, total_records_synced = ?6
             WHERE id = 1",
            params![
                state.last_sync_time,
                state.last_sync_record_count as i64,
                state.cc_switch_db_path,
                state.cc_switch_detected,
                state.sync_interval_secs as i64,
                state.total_records_synced as i64,
            ],
        )?;
        Ok(())
    }

    // ─── Budget Config ───────────────────────────────────────────────

    pub fn get_budget_config(&self) -> AppResult<BudgetConfig> {
        let conn = self.conn()?;
        let result = conn.query_row(
            "SELECT daily_limit_usd, weekly_limit_usd, monthly_limit_usd,
                    alert_threshold_pct, alert_icon_color, alert_system_notify
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
                })
            },
        )?;
        Ok(result)
    }

    pub fn update_budget_config(&self, config: &BudgetConfig) -> AppResult<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE budget_config SET
                daily_limit_usd = ?1, weekly_limit_usd = ?2, monthly_limit_usd = ?3,
                alert_threshold_pct = ?4, alert_icon_color = ?5, alert_system_notify = ?6
             WHERE id = 1",
            params![
                config.daily_limit_usd,
                config.weekly_limit_usd,
                config.monthly_limit_usd,
                config.alert_threshold_pct,
                config.alert_icon_color,
                config.alert_system_notify,
            ],
        )?;
        Ok(())
    }

    /// Check if any budget limits are exceeded (single-query optimization)
    pub fn check_budget_status(&self) -> AppResult<Option<crate::models::budget::BudgetAlert>> {
        let config = self.get_budget_config()?;
        let threshold = config.alert_threshold_pct as f64 / 100.0;

        // Collect all active periods and their limits in one pass
        let periods: Vec<(&str, f64)> = [
            config.daily_limit_usd.map(|l| ("today", l)),
            config.weekly_limit_usd.map(|l| ("week", l)),
            config.monthly_limit_usd.map(|l| ("month", l)),
        ]
        .into_iter()
        .flatten()
        .filter(|(_, l)| *l > 0.0)
        .collect();

        // Query costs for all active periods in a single connection acquisition
        let conn = self.conn()?;
        let mut costs: Vec<(&str, f64)> = Vec::new();
        for (period, limit) in &periods {
            let (start, end) = period_to_range(period)?;
            let cost: f64 = conn.query_row(
                "SELECT COALESCE(SUM(cost_usd), 0) FROM usage_records
                 WHERE timestamp >= ?1 AND timestamp < ?2",
                params![start, end],
                |row| row.get(0),
            )?;
            let pct = cost / limit;
            if pct >= threshold {
                costs.push((period, cost));
                // Return first triggered alert immediately (priority: daily > weekly > monthly)
                break;
            }
            costs.push((period, cost));
        }
        drop(conn);

        // Check which period triggered (if any)
        for (period, limit) in &periods {
            if let Some((_, cost)) = costs.iter().find(|(p, _)| p == period) {
                let pct = cost / limit;
                if pct >= threshold {
                    let period_label = match *period {
                        "today" => "Daily",
                        "week" => "Weekly",
                        "month" => "Monthly",
                        _ => "Unknown",
                    };
                    return Ok(Some(crate::models::budget::BudgetAlert {
                        triggered: true,
                        message: format!("{} budget {:.0}% used (${:.2} / ${:.2})", period_label, pct * 100.0, cost, limit),
                        current_cost_usd: *cost,
                        limit_usd: *limit,
                        percentage: pct * 100.0,
                        period: period.to_string(),
                    }));
                }
            }
        }

        Ok(None)
    }

    // ─── App Settings ────────────────────────────────────────────────

    pub fn get_app_settings(&self) -> AppResult<AppSettings> {
        let conn = self.conn()?;
        let result = conn.query_row(
            "SELECT language, download_source, auto_start, theme, tray_display,
                    telemetry_enabled, crash_log_enabled,
                    update_check_interval_hours
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
                    update_check_interval_hours: row.get(7)?,
                })
            },
        )?;
        Ok(result)
    }

    pub fn update_app_settings(&self, settings: &AppSettings) -> AppResult<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE app_settings SET
                language = ?1, download_source = ?2, auto_start = ?3,
                theme = ?4, tray_display = ?5, telemetry_enabled = ?6,
                crash_log_enabled = ?7,
                update_check_interval_hours = ?8
             WHERE id = 1",
            params![
                settings.language,
                settings.download_source,
                settings.auto_start,
                settings.theme,
                settings.tray_display,
                settings.telemetry_enabled,
                settings.crash_log_enabled,
                settings.update_check_interval_hours,
            ],
        )?;
        Ok(())
    }

    // ─── Custom Prices ───────────────────────────────────────────────

    pub fn get_custom_prices(&self) -> AppResult<Vec<ModelPricing>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT model_id, COALESCE(display_name, model_id),
                    input_per_million, output_per_million,
                    cache_write_per_million, cache_read_per_million,
                    reasoning_per_million,
                    created_at
             FROM custom_prices ORDER BY created_at DESC, model_id",
        )?;

        let result = stmt
            .query_map([], |row| {
                Ok(ModelPricing {
                    model_id: row.get(0)?,
                    display_name: row.get(1)?,
                    input_per_million: row.get(2)?,
                    output_per_million: row.get(3)?,
                    cache_write_per_million: row.get(4)?,
                    cache_read_per_million: row.get(5)?,
                    reasoning_per_million: row.get(6)?,
                    price_source: "custom".to_string(),
                    has_default: false,
                    created_at: row.get(7)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(result)
    }

    pub fn upsert_custom_price(&self, price: &ModelPricing) -> AppResult<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO custom_prices
                (model_id, display_name, input_per_million, output_per_million,
                 cache_write_per_million, cache_read_per_million, reasoning_per_million,
                 created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))
             ON CONFLICT(model_id) DO UPDATE SET
                display_name = excluded.display_name,
                input_per_million = excluded.input_per_million,
                output_per_million = excluded.output_per_million,
                cache_write_per_million = excluded.cache_write_per_million,
                cache_read_per_million = excluded.cache_read_per_million,
                reasoning_per_million = excluded.reasoning_per_million",
            params![
                price.model_id,
                price.display_name,
                price.input_per_million,
                price.output_per_million,
                price.cache_write_per_million,
                price.cache_read_per_million,
                price.reasoning_per_million,
            ],
        )?;
        Ok(())
    }

    pub fn delete_custom_price(&self, model_id: &str) -> AppResult<()> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM custom_prices WHERE model_id = ?1", params![model_id])?;
        Ok(())
    }

    /// Nullify cost_usd for all usage records of a specific model,
    /// so that backfill_costs will recalculate them with the latest price.
    /// Returns the number of affected records.
    pub fn invalidate_costs_for_model(&self, model_id: &str) -> AppResult<u64> {
        let conn = self.conn()?;
        let count = conn.execute(
            "UPDATE usage_records SET cost_usd = NULL WHERE model = ?1",
            params![model_id],
        )?;
        Ok(count as u64)
    }

    /// Count usage records for a specific model
    pub fn count_usage_records_for_model(&self, model_id: &str) -> AppResult<u64> {
        let conn = self.conn()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM usage_records WHERE model = ?1",
            params![model_id],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    // ─── Data Export ─────────────────────────────────────────────────

    pub fn export_usage_records(
        &self,
        period: &str,
    ) -> AppResult<Vec<UsageRecord>> {
        let (start, end) = period_to_range(period)?;
        let conn = self.conn()?;

        let mut stmt = conn.prepare(
            "SELECT id, source, session_id, timestamp, model,
                    input_tokens, output_tokens, cache_creation_tokens,
                    cache_read_tokens, total_tokens, reasoning_tokens,
                    cost_usd, project_path, provider_name,
                    response_time_ms, status_code, cc_switch_log_id
             FROM usage_records
             WHERE timestamp >= ?1 AND timestamp < ?2
             ORDER BY timestamp ASC",
        )?;

        let result = stmt
            .query_map(params![start, end], |row| {
                let source_str: String = row.get(1)?;
                let ts_str: String = row.get(3)?;
                Ok(UsageRecord {
                    id: row.get(0)?,
                    source: DataSource::from_str(&source_str)
                        .unwrap_or_else(|| {
                            log::warn!("Unknown data source in database: {}", source_str);
                            DataSource::CcSwitch
                        }),
                    session_id: row.get(2)?,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&ts_str)
                        .unwrap_or_else(|_| Utc::now().into())
                        .with_timezone(&Utc),
                    model: row.get(4)?,
                    tokens: TokenUsage {
                        input_tokens: row.get::<_, i64>(5)? as u64,
                        output_tokens: row.get::<_, i64>(6)? as u64,
                        cache_creation_tokens: row.get::<_, i64>(7)? as u64,
                        cache_read_tokens: row.get::<_, i64>(8)? as u64,
                        total_tokens: row.get::<_, i64>(9)? as u64,
                        reasoning_tokens: row.get::<_, i64>(10)? as u64,
                    },
                    cost_usd: row.get(11)?,
                    project_path: row.get(12)?,
                    provider_name: row.get(13)?,
                    response_time_ms: row.get::<_, Option<i64>>(14)?.map(|v| v as u64),
                    status_code: row.get::<_, Option<i64>>(15)?.map(|v| v as u16),
                    cc_switch_log_id: row.get(16)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(result)
    }

    // ─── Savings Engine Queries ────────────────────────────────────────

    /// Get cache efficiency metrics per data source for a given period.
    pub fn get_cache_efficiency(
        &self,
        period: &str,
        cache_savings: &HashMap<String, f64>,
    ) -> AppResult<Vec<CacheEfficiency>> {
        let (start, end) = period_to_range(period)?;
        let conn = self.conn()?;

        let mut stmt = conn.prepare(
            "SELECT source,
                    COALESCE(SUM(cache_read_tokens), 0),
                    COALESCE(SUM(cache_creation_tokens), 0),
                    COALESCE(SUM(input_tokens), 0),
                    COALESCE(SUM(cost_usd), 0)
             FROM usage_records
             WHERE timestamp >= ?1 AND timestamp < ?2
             GROUP BY source
             ORDER BY SUM(cost_usd) DESC",
        )?;

        let result = stmt
            .query_map(params![start, end], |row| {
                let source_str: String = row.get(0)?;
                let cache_read: i64 = row.get(1)?;
                let cache_creation: i64 = row.get(2)?;
                let input: i64 = row.get(3)?;
                let _cost: f64 = row.get(4)?;

                let display_name = DataSource::from_str(&source_str)
                    .map(|s| s.display_name().to_string())
                    .unwrap_or_else(|| source_str.clone());

                let cache_read_u64 = cache_read as u64;
                let input_u64 = input as u64;
                let total_for_hit = cache_read_u64 + input_u64;

                let hit_rate = if cache_read_u64 > 0 && total_for_hit > 0 {
                    Some(cache_read_u64 as f64 / total_for_hit as f64)
                } else {
                    None
                };

                let savings = cache_savings.get(&source_str).copied().unwrap_or(0.0);

                Ok(CacheEfficiency {
                    source: source_str,
                    display_name,
                    total_cache_read: cache_read_u64,
                    total_cache_creation: cache_creation as u64,
                    total_input: input_u64,
                    hit_rate,
                    cache_cost_savings: savings,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(result)
    }

    /// Compute per-source cache savings using actual model prices from the price table.
    pub fn get_cache_savings_by_source(
        &self,
        period: &str,
        prices: &HashMap<String, &ModelPricing>,
    ) -> AppResult<HashMap<String, f64>> {
        let (start, end) = period_to_range(period)?;
        let conn = self.conn()?;

        let mut stmt = conn.prepare(
            "SELECT source, model,
                    COALESCE(SUM(cache_read_tokens), 0)
             FROM usage_records
             WHERE timestamp >= ?1 AND timestamp < ?2
             GROUP BY source, model",
        )?;

        let mut result: HashMap<String, f64> = HashMap::new();

        let rows: Vec<(String, String, i64)> = stmt
            .query_map(params![start, end], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        for (source, model, cache_read) in rows {
            if cache_read <= 0 {
                continue;
            }

            let price = find_best_price(prices, &model);
            if let Some(p) = price {
                let input_price = p.input_per_million / 1_000_000.0;
                let cache_read_price =
                    p.cache_read_per_million.unwrap_or(p.input_per_million * 0.1) / 1_000_000.0;
                let saving = cache_read as f64 * (input_price - cache_read_price);
                *result.entry(source).or_insert(0.0) += saving;
            }
        }

        Ok(result)
    }

    /// Get model usage analysis with spending patterns
    pub fn get_model_usage_analysis(&self, period: &str) -> AppResult<ModelAnalysis> {
        let (start, end) = period_to_range(period)?;
        let conn = self.conn()?;

        let mut stmt = conn.prepare(
            "SELECT model, source,
                    COALESCE(SUM(cost_usd), 0),
                    COALESCE(SUM(total_tokens), 0),
                    COUNT(DISTINCT session_id)
             FROM usage_records
             WHERE timestamp >= ?1 AND timestamp < ?2
             GROUP BY model, source
             ORDER BY SUM(cost_usd) DESC",
        )?;

        let rows: Vec<(String, String, f64, i64, i64)> = stmt
            .query_map(params![start, end], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let total_cost: f64 = rows.iter().map(|(_, _, cost, _, _)| cost).sum();

        let insights: Vec<ModelUsageInsight> = rows
            .into_iter()
            .map(|(model, source, cost_usd, total_tokens, session_count)| {
                let tokens_u64 = total_tokens as u64;
                let sessions_u64 = session_count as u64;
                let cost_per_session = if sessions_u64 > 0 {
                    cost_usd / sessions_u64 as f64
                } else {
                    0.0
                };
                let cost_per_million_tokens = if tokens_u64 > 0 {
                    cost_usd / (tokens_u64 as f64 / 1_000_000.0)
                } else {
                    0.0
                };
                let cost_share_pct = if total_cost > 0.0 {
                    cost_usd / total_cost * 100.0
                } else {
                    0.0
                };

                ModelUsageInsight {
                    model,
                    source,
                    cost_usd,
                    total_tokens: tokens_u64,
                    session_count: sessions_u64,
                    cost_per_session,
                    cost_per_million_tokens,
                    cost_share_pct,
                }
            })
            .collect();

        let concentration_index: f64 = insights
            .iter()
            .map(|i| {
                let share = i.cost_share_pct / 100.0;
                share * share
            })
            .sum();

        let top_cost_model = insights.first().cloned();
        let top_cost_share_pct = top_cost_model
            .as_ref()
            .map(|m| m.cost_share_pct)
            .unwrap_or(0.0);

        Ok(ModelAnalysis {
            insights,
            top_cost_model,
            top_cost_share_pct,
            concentration_index,
        })
    }

    /// Detect cost anomalies over the last N days
    pub fn get_cost_anomalies(&self, days: u32, threshold: f64) -> AppResult<AnomalyReport> {
        let conn = self.conn()?;

        let start_date = Utc::now() - Duration::days(days as i64);
        let start_str = start_date.format("%Y-%m-%dT%H:%M:%S").to_string();

        let mut stmt = conn.prepare(
            "SELECT strftime('%Y-%m-%d', timestamp, 'localtime') as day,
                    COALESCE(SUM(cost_usd), 0)
             FROM usage_records
             WHERE timestamp >= ?1
             GROUP BY day
             ORDER BY day ASC",
        )?;

        let daily_costs: Vec<(String, f64)> = stmt
            .query_map(params![start_str], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        if daily_costs.is_empty() {
            return Ok(AnomalyReport {
                anomalies: vec![],
                daily_avg_cost: 0.0,
                daily_std_dev: 0.0,
                threshold_factor: threshold,
            });
        }

        let n = daily_costs.len() as f64;
        let mean: f64 = daily_costs.iter().map(|(_, c)| c).sum::<f64>() / n;
        let variance: f64 = daily_costs.iter().map(|(_, c)| (c - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();

        let anomaly_threshold = mean + threshold * std_dev;
        let mut anomalies: Vec<CostAnomaly> = daily_costs
            .iter()
            .filter(|(_, cost)| *cost > anomaly_threshold && std_dev > 0.0)
            .map(|(date, cost)| {
                let deviation_factor = if mean > 0.0 { cost / mean } else { 0.0 };
                CostAnomaly {
                    date: date.clone(),
                    cost_usd: *cost,
                    daily_avg: mean,
                    deviation_factor,
                    source: None,
                }
            })
            .collect();

        anomalies.sort_by(|a, b| b.deviation_factor.partial_cmp(&a.deviation_factor).unwrap_or(std::cmp::Ordering::Equal));

        for anomaly in anomalies.iter_mut().take(3) {
            let day_start = format!("{}T00:00:00", anomaly.date);
            let day_end = format!("{}T23:59:59", anomaly.date);

            if let Ok(top_source) = conn.query_row(
                "SELECT source FROM usage_records
                 WHERE timestamp >= ?1 AND timestamp <= ?2
                 GROUP BY source
                 ORDER BY SUM(cost_usd) DESC
                 LIMIT 1",
                params![day_start, day_end],
                |row| row.get::<_, String>(0),
            ) {
                let display_name = DataSource::from_str(&top_source)
                    .map(|s| s.display_name().to_string())
                    .unwrap_or(top_source);
                anomaly.source = Some(display_name);
            }
        }

        Ok(AnomalyReport {
            anomalies,
            daily_avg_cost: mean,
            daily_std_dev: std_dev,
            threshold_factor: threshold,
        })
    }

    /// Generate cost forecast for the current month
    pub fn get_cost_forecast(&self) -> AppResult<CostForecast> {
        use chrono::Local;
        let conn = self.conn()?;

        let local_now = Local::now();
        let first_of_month = local_now.date_naive().with_day(1).unwrap_or(local_now.date_naive());

        let next_month = if local_now.date_naive().month() == 12 {
            chrono::NaiveDate::from_ymd_opt(local_now.date_naive().year() + 1, 1, 1)
        } else {
            chrono::NaiveDate::from_ymd_opt(local_now.date_naive().year(), local_now.date_naive().month() + 1, 1)
        };
        let last_day = next_month
            .unwrap_or(first_of_month + Duration::days(31))
            .pred_opt()
            .unwrap_or(first_of_month);
        let total_days = (last_day - first_of_month).num_days() as u32 + 1;
        let day_of_month = local_now.date_naive().day0() as u32 + 1;
        let days_elapsed = day_of_month;
        let days_remaining = total_days.saturating_sub(days_elapsed);

        let month_start_utc = chrono::Local.from_local_datetime(
            &first_of_month.and_hms_opt(0, 0, 0).unwrap()
        ).single().unwrap_or_else(|| Local::now()).with_timezone(&Utc);
        let month_start_str = month_start_utc.format("%Y-%m-%dT%H:%M:%S").to_string();
        let now_str = Utc::now().to_rfc3339();

        let month_cost: f64 = conn.query_row(
            "SELECT COALESCE(SUM(cost_usd), 0) FROM usage_records
             WHERE timestamp >= ?1 AND timestamp < ?2",
            params![month_start_str, now_str],
            |row| row.get(0),
        )?;

        let daily_avg_cost = if days_elapsed > 0 {
            month_cost / days_elapsed as f64
        } else {
            0.0
        };

        let projected_monthly_cost = daily_avg_cost * total_days as f64;

        let monthly_limit = conn
            .query_row(
                "SELECT monthly_limit_usd FROM budget_config WHERE id = 1",
                [],
                |row| row.get::<_, Option<f64>>(0),
            )
            .unwrap_or(None)
            .filter(|l| *l > 0.0);

        let projected_over_budget = monthly_limit
            .map(|limit| projected_monthly_cost > limit)
            .unwrap_or(false);

        let budget_exhaustion_days = if let Some(limit) = monthly_limit {
            if daily_avg_cost > 0.0 {
                let remaining_budget = limit - month_cost;
                if remaining_budget > 0.0 {
                    Some((remaining_budget / daily_avg_cost).ceil() as u32)
                } else {
                    Some(0)
                }
            } else {
                None
            }
        } else {
            None
        };

        // Week-over-week change
        let week_start = {
            let weekday = local_now.date_naive().weekday();
            let days_since_monday = weekday.num_days_from_monday();
            local_now.date_naive() - Duration::days(days_since_monday as i64)
        };
        let prev_week_start = week_start - Duration::days(7);
        let prev_week_end = week_start;

        let week_start_utc = chrono::Local.from_local_datetime(
            &week_start.and_hms_opt(0, 0, 0).unwrap()
        ).single().unwrap_or_else(|| Local::now()).with_timezone(&Utc);
        let prev_week_start_utc = chrono::Local.from_local_datetime(
            &prev_week_start.and_hms_opt(0, 0, 0).unwrap()
        ).single().unwrap_or_else(|| Local::now()).with_timezone(&Utc);
        let prev_week_end_utc = chrono::Local.from_local_datetime(
            &prev_week_end.and_hms_opt(0, 0, 0).unwrap()
        ).single().unwrap_or_else(|| Local::now()).with_timezone(&Utc);

        let this_week_cost: f64 = conn.query_row(
            "SELECT COALESCE(SUM(cost_usd), 0) FROM usage_records
             WHERE timestamp >= ?1 AND timestamp < ?2",
            params![week_start_utc.format("%Y-%m-%dT%H:%M:%S").to_string(), now_str],
            |row| row.get(0),
        )?;

        let last_week_cost: f64 = conn.query_row(
            "SELECT COALESCE(SUM(cost_usd), 0) FROM usage_records
             WHERE timestamp >= ?1 AND timestamp < ?2",
            params![
                prev_week_start_utc.format("%Y-%m-%dT%H:%M:%S").to_string(),
                prev_week_end_utc.format("%Y-%m-%dT%H:%M:%S").to_string()
            ],
            |row| row.get(0),
        )?;

        let week_over_week_change_pct = if last_week_cost > 0.0 {
            Some((this_week_cost - last_week_cost) / last_week_cost * 100.0)
        } else {
            None
        };

        Ok(CostForecast {
            daily_avg_cost,
            projected_monthly_cost,
            days_remaining,
            days_elapsed,
            monthly_limit,
            projected_over_budget,
            budget_exhaustion_days,
            week_over_week_change_pct,
        })
    }

    // ─── Provider Analysis (CC Switch specific) ──────────────────────

    /// Get provider efficiency insights from CC Switch data
    pub fn get_provider_insights(&self, period: &str) -> AppResult<Vec<ProviderInsight>> {
        let (start, end) = period_to_range(period)?;
        let conn = self.conn()?;

        let mut stmt = conn.prepare(
            "SELECT provider_name,
                    COALESCE(SUM(cost_usd), 0),
                    COUNT(*) as request_count,
                    COALESCE(AVG(response_time_ms), 0),
                    SUM(CASE WHEN status_code IS NOT NULL AND status_code >= 400 THEN 1 ELSE 0 END) as failures
             FROM usage_records
             WHERE timestamp >= ?1 AND timestamp < ?2
               AND provider_name IS NOT NULL
             GROUP BY provider_name
             ORDER BY SUM(cost_usd) DESC",
        )?;

        let result = stmt
            .query_map(params![start, end], |row| {
                let total_cost: f64 = row.get(1)?;
                let request_count: i64 = row.get(2)?;
                let avg_latency: f64 = row.get(3)?;
                let failures: i64 = row.get(4)?;
                let req_u64 = request_count as u64;
                let failure_rate = if req_u64 > 0 {
                    failures as f64 / req_u64 as f64 * 100.0
                } else {
                    0.0
                };
                let cost_per_request = if req_u64 > 0 {
                    total_cost / req_u64 as f64
                } else {
                    0.0
                };

                Ok(ProviderInsight {
                    provider_name: row.get(0)?,
                    total_cost,
                    request_count: req_u64,
                    avg_latency_ms: avg_latency,
                    failure_rate_pct: failure_rate,
                    cost_per_request,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(result)
    }

    // ─── Analysis Cache ──────────────────────────────────────────────

    /// Get cached analysis result by key
    pub fn get_analysis_cache(&self, key: &str) -> AppResult<Option<String>> {
        let conn = self.conn()?;
        let result = conn.query_row(
            "SELECT value FROM analysis_cache
             WHERE key = ?1
               AND datetime(computed_at, '+' || ttl_secs || ' seconds') > datetime('now')",
            params![key],
            |row| row.get(0),
        );
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Set analysis cache result
    pub fn set_analysis_cache(&self, key: &str, value: &str, ttl_secs: u32) -> AppResult<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO analysis_cache (key, value, computed_at, ttl_secs)
             VALUES (?1, ?2, datetime('now'), ?3)",
            params![key, value, ttl_secs],
        )?;
        Ok(())
    }

    /// Get total record count
    pub fn get_total_record_count(&self) -> AppResult<u64> {
        let conn = self.conn()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM usage_records",
            [],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────

fn period_to_range(period: &str) -> AppResult<(String, String)> {
    use chrono::{Local, TimeZone};
    let now_utc = Utc::now();
    let start = match period {
        "today" => {
            let local_midnight = Local::now()
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let local_dt = Local.from_local_datetime(&local_midnight)
                .single()
                .unwrap_or_else(|| Local::now());
            local_dt.with_timezone(&Utc)
        }
        "week" => {
            let local_now = Local::now();
            let weekday = local_now.date_naive().weekday();
            let days_since_monday = weekday.num_days_from_monday();
            let monday = local_now.date_naive() - Duration::days(days_since_monday as i64);
            let monday_midnight = monday.and_hms_opt(0, 0, 0).unwrap();
            let local_dt = Local.from_local_datetime(&monday_midnight)
                .single()
                .unwrap_or_else(|| Local::now());
            local_dt.with_timezone(&Utc)
        }
        "month" => {
            let local_now = Local::now();
            let first_of_month = local_now.date_naive().with_day(1).unwrap_or(local_now.date_naive());
            let month_midnight = first_of_month.and_hms_opt(0, 0, 0).unwrap();
            let local_dt = Local.from_local_datetime(&month_midnight)
                .single()
                .unwrap_or_else(|| Local::now());
            local_dt.with_timezone(&Utc)
        }
        "all" => now_utc - Duration::days(365 * 20),
        _ => now_utc - Duration::days(30),
    };
    Ok((
        start.format("%Y-%m-%dT%H:%M:%S").to_string(),
        now_utc.to_rfc3339(),
    ))
}

/// Find the best matching price for a model name from a price map.
/// Match priority: exact > case-insensitive exact > prefix (longest match wins).
fn find_best_price<'a>(
    prices: &'a HashMap<String, &ModelPricing>,
    model: &str,
) -> Option<&'a ModelPricing> {
    // Try exact match first
    if let Some(price) = prices.get(model) {
        return Some(price);
    }

    // Try case-insensitive exact match
    let model_lower = model.to_lowercase();
    for (key, price) in prices.iter() {
        if key.to_lowercase() == model_lower {
            return Some(price);
        }
    }

    // Prefix match: model name starts with the price key (case-insensitive)
    let mut best_match: Option<(&String, &&ModelPricing)> = None;
    for (key, price) in prices.iter() {
        if key.len() >= 4 && model_lower.starts_with(&key.to_lowercase()) {
            match &best_match {
                None => best_match = Some((key, price)),
                Some((best_key, _)) if key.len() > best_key.len() => {
                    best_match = Some((key, price));
                }
                _ => {}
            }
        }
    }

    best_match.map(|(_, price)| *price)
}
