use rusqlite::Connection;
use crate::error::AppResult;

/// Current schema version
pub const CURRENT_SCHEMA_VERSION: i64 = 2;

/// Ensure the schema_version table exists and return the current version (0 if none)
fn ensure_schema_version_table(conn: &Connection) -> AppResult<i64> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);",
    )?;
    let version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(version)
}

/// Set the schema version to the given value
fn set_schema_version(conn: &Connection, version: i64) -> AppResult<()> {
    conn.execute("DELETE FROM schema_version", [])?;
    conn.execute("INSERT INTO schema_version (version) VALUES (?1)", [version])?;
    Ok(())
}

pub fn run_all(conn: &Connection) -> AppResult<()> {
    let current_version = ensure_schema_version_table(conn)?;

    // Check if critical tables exist (safety check for corrupted databases)
    let tables_exist = check_critical_tables_exist(conn)?;

    // If old schema exists or critical tables are missing, rebuild from scratch
    if (current_version > 0 && current_version != CURRENT_SCHEMA_VERSION) || !tables_exist {
        if !tables_exist && current_version > 0 {
            log::warn!(
                "Critical tables missing despite schema version {}. Rebuilding...",
                current_version
            );
        } else {
            log::warn!(
                "Old schema version {} detected. Dropping all tables and rebuilding...",
                current_version
            );
        }
        drop_all_tables(conn)?;
    }

    let effective_version = ensure_schema_version_table(conn)?;
    if effective_version < CURRENT_SCHEMA_VERSION {
        run_v1(conn)?;
        set_schema_version(conn, CURRENT_SCHEMA_VERSION)?;
    }

    Ok(())
}

/// Check if critical tables exist (usage_records, app_settings)
fn check_critical_tables_exist(conn: &Connection) -> AppResult<bool> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('usage_records', 'app_settings')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(count == 2)
}

/// Drop all existing tables (used when migrating from old schema)
fn drop_all_tables(conn: &Connection) -> AppResult<()> {
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")?
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    for table in &tables {
        conn.execute(&format!("DROP TABLE IF EXISTS \"{}\"", table), [])?;
        log::info!("Dropped old table: {}", table);
    }

    // Also drop indexes
    let indexes: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'")?
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    for index in &indexes {
        conn.execute(&format!("DROP INDEX IF EXISTS \"{}\"", index), [])?;
    }

    Ok(())
}

/// v1: Fresh schema for CC Switch companion product
fn run_v1(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "
        -- Core usage records (synced from CC Switch proxy_request_logs)
        CREATE TABLE IF NOT EXISTS usage_records (
            id TEXT PRIMARY KEY,
            source TEXT NOT NULL DEFAULT 'ccswitch',
            session_id TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            model TEXT NOT NULL,
            input_tokens INTEGER NOT NULL DEFAULT 0,
            output_tokens INTEGER NOT NULL DEFAULT 0,
            cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
            cache_read_tokens INTEGER NOT NULL DEFAULT 0,
            total_tokens INTEGER NOT NULL DEFAULT 0,
            reasoning_tokens INTEGER NOT NULL DEFAULT 0,
            cost_usd REAL,
            project_path TEXT,
            provider_name TEXT,
            response_time_ms INTEGER,
            status_code INTEGER,
            cc_switch_log_id TEXT UNIQUE,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Query optimization indexes
        CREATE INDEX IF NOT EXISTS idx_records_timestamp
            ON usage_records(timestamp);
        CREATE INDEX IF NOT EXISTS idx_records_model_timestamp
            ON usage_records(model, timestamp);
        CREATE INDEX IF NOT EXISTS idx_records_provider
            ON usage_records(provider_name, timestamp);
        CREATE INDEX IF NOT EXISTS idx_records_ts_cost
            ON usage_records(timestamp, cost_usd);
        CREATE INDEX IF NOT EXISTS idx_records_session
            ON usage_records(session_id);

        -- Sync state between TokenOwl and CC Switch (single row)
        CREATE TABLE IF NOT EXISTS sync_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            last_sync_time TEXT,
            last_sync_record_count INTEGER DEFAULT 0,
            cc_switch_db_path TEXT,
            cc_switch_detected BOOLEAN DEFAULT 0,
            sync_interval_secs INTEGER DEFAULT 300,
            total_records_synced INTEGER DEFAULT 0
        );
        INSERT OR IGNORE INTO sync_state (id) VALUES (1);

        -- Budget configuration (single row)
        CREATE TABLE IF NOT EXISTS budget_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            daily_limit_usd REAL,
            weekly_limit_usd REAL,
            monthly_limit_usd REAL,
            alert_threshold_pct INTEGER DEFAULT 80,
            alert_icon_color BOOLEAN DEFAULT 1,
            alert_system_notify BOOLEAN DEFAULT 1
        );
        INSERT OR IGNORE INTO budget_config (id) VALUES (1);

        -- User custom model prices (overrides for CC Switch cost data)
        CREATE TABLE IF NOT EXISTS custom_prices (
            model_id TEXT PRIMARY KEY,
            display_name TEXT,
            input_per_million REAL NOT NULL,
            output_per_million REAL NOT NULL,
            cache_write_per_million REAL,
            cache_read_per_million REAL,
            reasoning_per_million REAL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Application settings (single row)
        CREATE TABLE IF NOT EXISTS app_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            language TEXT DEFAULT 'auto',
            download_source TEXT DEFAULT 'auto',
            auto_start BOOLEAN DEFAULT 0,
            theme TEXT DEFAULT 'system',
            tray_display TEXT DEFAULT 'cost',
            telemetry_enabled BOOLEAN DEFAULT 0,
            crash_log_enabled BOOLEAN DEFAULT 1,
            update_check_interval_hours INTEGER DEFAULT 4,
            last_update_check TEXT
        );
        INSERT OR IGNORE INTO app_settings (id) VALUES (1);

        -- Crash logs
        CREATE TABLE IF NOT EXISTS crash_logs (
            id TEXT PRIMARY KEY,
            timestamp TEXT NOT NULL,
            error_type TEXT NOT NULL,
            message TEXT NOT NULL,
            stack_trace TEXT,
            app_version TEXT NOT NULL,
            os_info TEXT NOT NULL,
            context TEXT DEFAULT '{}'
        );

        -- Analysis result cache (avoid recomputation)
        CREATE TABLE IF NOT EXISTS analysis_cache (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            computed_at TEXT NOT NULL,
            ttl_secs INTEGER DEFAULT 3600
        );
        ",
    )?;

    Ok(())
}
