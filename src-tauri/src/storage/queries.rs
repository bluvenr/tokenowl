use chrono::{Duration, TimeZone, Utc};
use rusqlite::params;

use crate::error::AppResult;
use crate::models::usage::*;
use crate::models::budget::BudgetConfig;
use crate::models::settings::{AppSettings, ModelPricing, SourceConfig};
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
                     cost_usd, project_path)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
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

    /// Atomically replace all records for a session: delete old + insert new in a transaction
    pub fn replace_session_records(
        &self,
        source: &str,
        session_id: &str,
        records: &[UsageRecord],
    ) -> AppResult<usize> {
        let mut conn = self.conn()?;
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM usage_records WHERE source = ?1 AND session_id = ?2",
            params![source, session_id],
        )?;
        let mut count = 0;
        for r in records {
            tx.execute(
                "INSERT OR IGNORE INTO usage_records
                    (id, source, session_id, timestamp, model,
                     input_tokens, output_tokens, cache_creation_tokens,
                     cache_read_tokens, total_tokens, reasoning_tokens,
                     cost_usd, project_path)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
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
                ],
            )?;
            if tx.changes() > 0 {
                count += 1;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// Count records for a specific source
    pub fn count_source_records(&self, source: &str) -> AppResult<u64> {
        let conn = self.conn()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM usage_records WHERE source = ?1",
            params![source],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    /// Get usage summary for a period
    pub fn get_usage_summary(&self, period: &str) -> AppResult<UsageSummary> {
        let (start, _end) = period_to_range(period)?;
        let conn = self.conn()?;

        let result = conn.query_row(
            "SELECT COALESCE(SUM(cost_usd), 0),
                    COALESCE(SUM(total_tokens), 0),
                    COALESCE(SUM(input_tokens), 0),
                    COALESCE(SUM(output_tokens), 0),
                    COALESCE(SUM(reasoning_tokens), 0),
                    COUNT(DISTINCT source || '|' || session_id)
             FROM usage_records
             WHERE timestamp >= ?1",
            params![start],
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

    /// Get usage breakdown by source
    pub fn get_usage_by_source(&self, period: &str) -> AppResult<Vec<SourceUsage>> {
        let (start, _end) = period_to_range(period)?;
        let conn = self.conn()?;

        let mut stmt = conn.prepare(
            "SELECT source,
                    COALESCE(SUM(cost_usd), 0) as cost,
                    COALESCE(SUM(total_tokens), 0) as tokens
             FROM usage_records
             WHERE timestamp >= ?1
             GROUP BY source
             ORDER BY cost DESC",
        )?;

        let rows: Vec<(String, f64, i64)> = stmt
            .query_map(params![start], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let total_cost: f64 = rows.iter().map(|(_, cost, _)| cost).sum();

        let result = rows
            .into_iter()
            .map(|(source, cost_usd, total_tokens)| {
                let display_name = DataSource::from_str(&source)
                    .map(|s| s.display_name().to_string())
                    .unwrap_or_else(|| source.clone());
                let percentage = if total_cost > 0.0 {
                    cost_usd / total_cost * 100.0
                } else {
                    0.0
                };
                SourceUsage {
                    source,
                    display_name,
                    cost_usd,
                    total_tokens: total_tokens as u64,
                    percentage,
                }
            })
            .collect();

        Ok(result)
    }

    /// Get usage breakdown by model
    pub fn get_usage_by_model(&self, period: &str) -> AppResult<Vec<ModelUsage>> {
        let (start, _end) = period_to_range(period)?;
        let conn = self.conn()?;

        let mut stmt = conn.prepare(
            "SELECT model, source,
                    COALESCE(SUM(cost_usd), 0),
                    COALESCE(SUM(total_tokens), 0),
                    COALESCE(SUM(input_tokens), 0),
                    COALESCE(SUM(output_tokens), 0),
                    COALESCE(SUM(reasoning_tokens), 0)
             FROM usage_records
             WHERE timestamp >= ?1
             GROUP BY model, source
             ORDER BY SUM(cost_usd) DESC",
        )?;

        let result = stmt
            .query_map(params![start], |row| {
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
        let (start, _end) = period_to_range(period)?;
        let conn = self.conn()?;

        let date_format = match granularity {
            "hourly" => "%Y-%m-%d %H:00",
            "daily" => "%Y-%m-%d",
            "weekly" => "%Y-W%W",
            _ => "%Y-%m-%d",
        };

        let mut stmt = conn.prepare(&format!(
            "SELECT strftime('{}', timestamp) as date_bucket,
                    COALESCE(SUM(cost_usd), 0),
                    COALESCE(SUM(total_tokens), 0)
             FROM usage_records
             WHERE timestamp >= ?1
             GROUP BY date_bucket
             ORDER BY date_bucket ASC",
            date_format
        ))?;

        let result = stmt
            .query_map(params![start], |row| {
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

    // ─── File Offsets ────────────────────────────────────────────────

    pub fn get_file_offset(&self, file_path: &str) -> AppResult<u64> {
        let conn = self.conn()?;
        let offset: i64 = conn.query_row(
            "SELECT byte_offset FROM file_offsets WHERE file_path = ?1",
            params![file_path],
            |row| row.get(0),
        ).unwrap_or(0);
        Ok(offset as u64)
    }

    pub fn set_file_offset(&self, file_path: &str, source: &str, offset: u64) -> AppResult<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO file_offsets (file_path, source, byte_offset, last_modified)
             VALUES (?1, ?2, ?3, datetime('now'))",
            params![file_path, source, offset],
        )?;
        Ok(())
    }

    /// Get all records that don't have a cost_usd value (for cost backfill)
    pub fn get_records_without_cost(&self, limit: u32) -> AppResult<Vec<UsageRecord>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, source, session_id, timestamp, model,
                    input_tokens, output_tokens, cache_creation_tokens,
                    cache_read_tokens, total_tokens, reasoning_tokens, project_path
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
                            DataSource::ClaudeCode
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

    /// Check if any budget limits are exceeded
    pub fn check_budget_status(&self) -> AppResult<Option<crate::models::budget::BudgetAlert>> {
        let config = self.get_budget_config()?;
        let threshold = config.alert_threshold_pct as f64 / 100.0;

        // Check daily
        if let Some(limit) = config.daily_limit_usd {
            if limit > 0.0 {
                let summary = self.get_usage_summary("today")?;
                let pct = summary.total_cost_usd / limit;
                if pct >= threshold {
                    return Ok(Some(crate::models::budget::BudgetAlert {
                        triggered: true,
                        message: format!("Daily budget {:.0}% used (${:.2} / ${:.2})", pct * 100.0, summary.total_cost_usd, limit),
                        current_cost_usd: summary.total_cost_usd,
                        limit_usd: limit,
                        percentage: pct * 100.0,
                        period: "today".to_string(),
                    }));
                }
            }
        }

        // Check weekly
        if let Some(limit) = config.weekly_limit_usd {
            if limit > 0.0 {
                let summary = self.get_usage_summary("week")?;
                let pct = summary.total_cost_usd / limit;
                if pct >= threshold {
                    return Ok(Some(crate::models::budget::BudgetAlert {
                        triggered: true,
                        message: format!("Weekly budget {:.0}% used (${:.2} / ${:.2})", pct * 100.0, summary.total_cost_usd, limit),
                        current_cost_usd: summary.total_cost_usd,
                        limit_usd: limit,
                        percentage: pct * 100.0,
                        period: "week".to_string(),
                    }));
                }
            }
        }

        // Check monthly
        if let Some(limit) = config.monthly_limit_usd {
            if limit > 0.0 {
                let summary = self.get_usage_summary("month")?;
                let pct = summary.total_cost_usd / limit;
                if pct >= threshold {
                    return Ok(Some(crate::models::budget::BudgetAlert {
                        triggered: true,
                        message: format!("Monthly budget {:.0}% used (${:.2} / ${:.2})", pct * 100.0, summary.total_cost_usd, limit),
                        current_cost_usd: summary.total_cost_usd,
                        limit_usd: limit,
                        percentage: pct * 100.0,
                        period: "month".to_string(),
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
                    price_sync_interval_hours, update_check_interval_hours
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
                    price_sync_interval_hours: row.get(7)?,
                    update_check_interval_hours: row.get(8)?,
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
                crash_log_enabled = ?7, price_sync_interval_hours = ?8,
                update_check_interval_hours = ?9
             WHERE id = 1",
            params![
                settings.language,
                settings.download_source,
                settings.auto_start,
                settings.theme,
                settings.tray_display,
                settings.telemetry_enabled,
                settings.crash_log_enabled,
                settings.price_sync_interval_hours,
                settings.update_check_interval_hours,
            ],
        )?;
        Ok(())
    }

    // ─── Source Config ───────────────────────────────────────────────

    pub fn get_source_configs(&self) -> AppResult<Vec<SourceConfig>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT source, enabled, custom_path FROM source_config ORDER BY source",
        )?;

        let result = stmt
            .query_map([], |row| {
                Ok(SourceConfig {
                    source: row.get(0)?,
                    enabled: row.get(1)?,
                    custom_path: row.get(2)?,
                    available: false,
                    status: "unavailable".to_string(),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(result)
    }

    pub fn update_source_config(&self, source: &str, enabled: bool, custom_path: Option<&str>) -> AppResult<()> {
        let conn = self.conn()?;
        conn.execute(
            "UPDATE source_config SET enabled = ?2, custom_path = ?3 WHERE source = ?1",
            params![source, enabled, custom_path],
        )?;
        Ok(())
    }

    // ─── Custom Prices ───────────────────────────────────────────────

    pub fn get_custom_prices(&self) -> AppResult<Vec<ModelPricing>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT model_id, COALESCE(display_name, model_id), COALESCE(source, ''),
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
                    source: row.get(2)?,
                    input_per_million: row.get(3)?,
                    output_per_million: row.get(4)?,
                    cache_write_per_million: row.get(5)?,
                    cache_read_per_million: row.get(6)?,
                    reasoning_per_million: row.get(7)?,
                    price_source: "custom".to_string(),
                    has_default: false, // computed later in merge_prices
                    created_at: row.get(8)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(result)
    }

    pub fn upsert_custom_price(&self, price: &ModelPricing) -> AppResult<()> {
        let conn = self.conn()?;
        // Use ON CONFLICT to preserve created_at on update
        conn.execute(
            "INSERT INTO custom_prices
                (model_id, display_name, source, input_per_million, output_per_million,
                 cache_write_per_million, cache_read_per_million, reasoning_per_million,
                 created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))
             ON CONFLICT(model_id) DO UPDATE SET
                display_name = excluded.display_name,
                source = excluded.source,
                input_per_million = excluded.input_per_million,
                output_per_million = excluded.output_per_million,
                cache_write_per_million = excluded.cache_write_per_million,
                cache_read_per_million = excluded.cache_read_per_million,
                reasoning_per_million = excluded.reasoning_per_million",
            params![
                price.model_id,
                price.display_name,
                price.source,
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
        let (start, _end) = period_to_range(period)?;
        let conn = self.conn()?;

        let mut stmt = conn.prepare(
            "SELECT id, source, session_id, timestamp, model,
                    input_tokens, output_tokens, cache_creation_tokens,
                    cache_read_tokens, total_tokens, reasoning_tokens,
                    cost_usd, project_path
             FROM usage_records
             WHERE timestamp >= ?1
             ORDER BY timestamp ASC",
        )?;

        let result = stmt
            .query_map(params![start], |row| {
                let source_str: String = row.get(1)?;
                let ts_str: String = row.get(3)?;
                Ok(UsageRecord {
                    id: row.get(0)?,
                    source: DataSource::from_str(&source_str)
                        .unwrap_or_else(|| {
                            log::warn!("Unknown data source in database: {}", source_str);
                            DataSource::ClaudeCode
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
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(result)
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────

fn period_to_range(period: &str) -> AppResult<(String, String)> {
    use chrono::Local;
    let now_utc = Utc::now();
    let start = match period {
        "today" => {
            // Get midnight in the user's local timezone, then convert to UTC
            let local_midnight = Local::now()
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let local_dt = chrono::Local.from_local_datetime(&local_midnight)
                .single()
                .unwrap_or_else(|| Local::now());
            local_dt.with_timezone(&Utc)
        }
        "week" => now_utc - Duration::days(7),
        "month" => now_utc - Duration::days(30),
        "all" => now_utc - Duration::days(365 * 20),
        _ => now_utc - Duration::days(30),
    };
    Ok((
        start.format("%Y-%m-%dT%H:%M:%S").to_string(),
        now_utc.to_rfc3339(),
    ))
}
