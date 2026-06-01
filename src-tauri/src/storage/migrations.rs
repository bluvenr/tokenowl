use rusqlite::Connection;
use crate::error::AppResult;

/// Current schema version — bump this when adding new migrations
pub const CURRENT_SCHEMA_VERSION: i64 = 1;

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

    // Version 1: initial schema (current tables)
    if current_version < 1 {
        run_v1(conn)?;
        set_schema_version(conn, 1)?;
    }

    // Future migrations go here:
    // if current_version < 2 { run_v2(conn)?; set_schema_version(conn, 2)?; }

    Ok(())
}

fn run_v1(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "
        -- Core usage records table
        CREATE TABLE IF NOT EXISTS usage_records (
            id TEXT PRIMARY KEY,
            source TEXT NOT NULL,
            session_id TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            model TEXT NOT NULL,
            input_tokens INTEGER NOT NULL DEFAULT 0,
            output_tokens INTEGER NOT NULL DEFAULT 0,
            cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
            cache_read_tokens INTEGER NOT NULL DEFAULT 0,
            total_tokens INTEGER NOT NULL DEFAULT 0,
            cost_usd REAL,
            project_path TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Query optimization indexes
        CREATE INDEX IF NOT EXISTS idx_source_timestamp ON usage_records(source, timestamp);
        CREATE INDEX IF NOT EXISTS idx_model_timestamp ON usage_records(model, timestamp);
        CREATE INDEX IF NOT EXISTS idx_session ON usage_records(session_id);

        -- File offset tracking for incremental parsing
        CREATE TABLE IF NOT EXISTS file_offsets (
            file_path TEXT PRIMARY KEY,
            source TEXT NOT NULL,
            byte_offset INTEGER NOT NULL DEFAULT 0,
            last_modified TEXT
        );

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

        -- Custom model prices (user overrides)
        CREATE TABLE IF NOT EXISTS custom_prices (
            model_id TEXT PRIMARY KEY,
            display_name TEXT,
            source TEXT,
            input_per_million REAL NOT NULL,
            output_per_million REAL NOT NULL,
            cache_write_per_million REAL,
            cache_read_per_million REAL
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
            price_sync_interval_hours INTEGER DEFAULT 12,
            update_check_interval_hours INTEGER DEFAULT 4,
            last_price_sync TEXT,
            last_update_check TEXT
        );

        -- Data source configuration (one row per source)
        CREATE TABLE IF NOT EXISTS source_config (
            source TEXT PRIMARY KEY,
            enabled BOOLEAN DEFAULT 1,
            custom_path TEXT
        );
        ",
    )?;

    // Ensure default rows exist
    conn.execute_batch(
        "
        INSERT OR IGNORE INTO budget_config (id) VALUES (1);
        INSERT OR IGNORE INTO app_settings (id) VALUES (1);

        INSERT OR IGNORE INTO source_config (source, enabled) VALUES ('claude_code', 1);
        INSERT OR IGNORE INTO source_config (source, enabled) VALUES ('codex_cli', 1);
        INSERT OR IGNORE INTO source_config (source, enabled) VALUES ('gemini_cli', 1);
        INSERT OR IGNORE INTO source_config (source, enabled) VALUES ('kimi_code', 1);
        INSERT OR IGNORE INTO source_config (source, enabled) VALUES ('qwen_code', 1);
        ",
    )?;

    Ok(())
}
