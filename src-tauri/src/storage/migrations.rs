use rusqlite::{Connection, Result};

/// Initialize database schema.
/// Uses IF NOT EXISTS for idempotent initialization on fresh databases.
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // usage_records - main API usage tracking table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS usage_records (
            id TEXT PRIMARY KEY,
            timestamp TEXT NOT NULL,
            app_type TEXT NOT NULL,
            provider_name TEXT,
            model TEXT NOT NULL,
            input_tokens INTEGER NOT NULL DEFAULT 0,
            output_tokens INTEGER NOT NULL DEFAULT 0,
            cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
            cache_read_tokens INTEGER NOT NULL DEFAULT 0,
            reasoning_tokens INTEGER NOT NULL DEFAULT 0,
            total_tokens INTEGER NOT NULL DEFAULT 0,
            cost_usd REAL,
            status_code INTEGER,
            response_time_ms INTEGER,
            cc_switch_log_id TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_provider_timestamp ON usage_records(provider_name, timestamp)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_model_timestamp ON usage_records(model, timestamp)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_app_type_timestamp ON usage_records(app_type, timestamp)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ccswitch_log ON usage_records(cc_switch_log_id)",
        [],
    )?;

    // sync_state - CC Switch sync tracking
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sync_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            last_sync_time TEXT,
            last_sync_record_count INTEGER DEFAULT 0,
            cc_switch_db_path TEXT,
            cc_switch_detected BOOLEAN DEFAULT 0,
            sync_interval_secs INTEGER DEFAULT 300,
            total_records_synced INTEGER DEFAULT 0
        )",
        [],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO sync_state (id) VALUES (1)",
        [],
    )?;

    // analysis_cache - computed analysis results cache
    conn.execute(
        "CREATE TABLE IF NOT EXISTS analysis_cache (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            computed_at TEXT NOT NULL,
            ttl_secs INTEGER DEFAULT 3600
        )",
        [],
    )?;

    // budget_config - budget and alert settings
    conn.execute(
        "CREATE TABLE IF NOT EXISTS budget_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            daily_limit_usd REAL,
            weekly_limit_usd REAL,
            monthly_limit_usd REAL,
            alert_threshold_pct INTEGER DEFAULT 80,
            alert_icon_color BOOLEAN DEFAULT 1,
            alert_system_notify BOOLEAN DEFAULT 1,
            alert_dashboard_banner BOOLEAN DEFAULT 1
        )",
        [],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO budget_config (id) VALUES (1)",
        [],
    )?;

    // custom_prices - user-defined model pricing overrides
    conn.execute(
        "CREATE TABLE IF NOT EXISTS custom_prices (
            model_id TEXT PRIMARY KEY,
            input_per_million REAL NOT NULL,
            output_per_million REAL NOT NULL,
            cache_write_per_million REAL,
            cache_read_per_million REAL
        )",
        [],
    )?;

    // app_settings - application settings (singleton)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            language TEXT DEFAULT 'auto',
            download_source TEXT DEFAULT 'auto',
            auto_start BOOLEAN DEFAULT 0,
            theme TEXT DEFAULT 'system',
            tray_display TEXT DEFAULT 'cost',
            telemetry_enabled BOOLEAN DEFAULT 0,
            crash_log_enabled BOOLEAN DEFAULT 1,
            anomaly_threshold REAL DEFAULT 2.5,
            forecast_method TEXT DEFAULT 'linear',
            data_retention_days INTEGER DEFAULT 90,
            daily_digest_enabled BOOLEAN DEFAULT 0,
            daily_digest_time TEXT DEFAULT '20:00',
            weekly_digest_enabled BOOLEAN DEFAULT 0,
            update_check_interval_hours INTEGER DEFAULT 4,
            price_sync_interval_hours INTEGER DEFAULT 12,
            default_period TEXT DEFAULT 'week'
        )",
        [],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO app_settings (id) VALUES (1)",
        [],
    )?;

    // crash_logs - application crash records
    conn.execute(
        "CREATE TABLE IF NOT EXISTS crash_logs (
            id TEXT PRIMARY KEY,
            timestamp TEXT NOT NULL,
            app_version TEXT,
            os TEXT,
            arch TEXT,
            error_type TEXT,
            message TEXT,
            backtrace TEXT,
            context TEXT
        )",
        [],
    )?;

    Ok(())
}
