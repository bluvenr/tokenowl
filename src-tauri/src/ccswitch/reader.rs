use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::{AppError, AppResult};
use crate::models::usage::{CcSwitchInfo, DataSource, TokenUsage, UsageRecord};

/// Default CC Switch database path: ~/.cc-switch/cc-switch.db
const CC_SWITCH_DB_NAME: &str = "cc-switch.db";
const CC_SWITCH_DIR: &str = ".cc-switch";

/// CC Switch database table name
const PROXY_REQUEST_LOGS: &str = "proxy_request_logs";

/// A raw record from CC Switch's proxy_request_logs table
/// Schema matches CC Switch actual database structure
struct CcSwitchLogRow {
    pub request_id: String,
    pub session_id: Option<String>,
    pub created_at: i64,           // Unix timestamp (milliseconds)
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cache_read_tokens: i64,
    pub total_cost_usd: Option<f64>,
    pub provider_id: String,
    pub latency_ms: Option<i64>,
    pub status_code: i64,
}

/// Read-only connection to a CC Switch SQLite database
pub struct CcSwitchReader {
    db_path: PathBuf,
    conn: Mutex<Option<Connection>>,
}

impl CcSwitchReader {
    pub fn new() -> Self {
        Self {
            db_path: Self::default_db_path(),
            conn: Mutex::new(None),
        }
    }

    /// Create with a specific database path (for testing or custom installations)
    pub fn with_path(db_path: PathBuf) -> Self {
        Self {
            db_path,
            conn: Mutex::new(None),
        }
    }

    /// Default path: ~/.cc-switch/cc-switch.db
    pub fn default_db_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(CC_SWITCH_DIR)
            .join(CC_SWITCH_DB_NAME)
    }

    /// Try to detect CC Switch installation by checking if the database file exists
    pub fn detect(&self) -> bool {
        self.db_path.exists()
    }

    /// Get connection info about the detected CC Switch database
    pub fn get_info(&self) -> AppResult<CcSwitchInfo> {
        if !self.db_path.exists() {
            return Err(AppError::NotFound("CC Switch database not found".to_string()));
        }

        let db_size = std::fs::metadata(&self.db_path)
            .map(|m| m.len())
            .unwrap_or(0);

        let record_count = self.get_record_count().unwrap_or(0);

        Ok(CcSwitchInfo {
            db_path: self.db_path.to_string_lossy().to_string(),
            db_size_bytes: db_size,
            record_count,
        })
    }

    /// Get the database path
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// Update the database path (e.g., user manually specified a different location)
    pub fn set_db_path(&mut self, path: PathBuf) {
        // Close existing connection
        if let Ok(mut guard) = self.conn.lock() {
            *guard = None;
        }
        self.db_path = path;
    }

    /// Open a read-only connection to the CC Switch database
    fn open_connection(&self) -> AppResult<()> {
        let mut guard = self.conn.lock()
            .map_err(|e| AppError::Config(format!("CC Switch reader mutex poisoned: {}", e)))?;

        if guard.is_some() {
            // Connection already open, verify it's still valid
            if let Some(ref conn) = *guard {
                if conn.query_row("SELECT 1", [], |_| Ok(())).is_ok() {
                    return Ok(());
                }
            }
        }

        if !self.db_path.exists() {
            return Err(AppError::NotFound(
                format!("CC Switch database not found at: {}", self.db_path.display())
            ));
        }

        let conn = Connection::open_with_flags(
            &self.db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        // Set read-only pragmas for safety
        conn.execute_batch("PRAGMA query_only=ON;")?;

        *guard = Some(conn);
        Ok(())
    }

    /// Get total record count in CC Switch database
    pub fn get_record_count(&self) -> AppResult<u64> {
        self.open_connection()?;
        let guard = self.conn.lock()
            .map_err(|e| AppError::Config(format!("CC Switch reader mutex poisoned: {}", e)))?;
        let conn = guard.as_ref().ok_or_else(|| {
            AppError::Config("CC Switch connection not available".to_string())
        })?;

        let count: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM {}", PROXY_REQUEST_LOGS),
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        Ok(count as u64)
    }

    /// Read new records since the last synced timestamp.
    /// Returns records ordered by created_at ASC.
    pub fn read_new_records(&self, last_synced_ts: i64, limit: u32) -> AppResult<Vec<UsageRecord>> {
        self.open_connection()?;
        let guard = self.conn.lock()
            .map_err(|e| AppError::Config(format!("CC Switch reader mutex poisoned: {}", e)))?;
        let conn = guard.as_ref().ok_or_else(|| {
            AppError::Config("CC Switch connection not available".to_string())
        })?;

        // Check if the table exists
        let table_exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
            params![PROXY_REQUEST_LOGS],
            |row| row.get::<_, i64>(0),
        ).unwrap_or(0) > 0;

        if !table_exists {
            return Err(AppError::Sync {
                origin: "ccswitch".to_string(),
                message: format!("Table '{}' not found in CC Switch database", PROXY_REQUEST_LOGS),
            });
        }

        let mut stmt = conn.prepare(
            &format!(
                "SELECT request_id, session_id, created_at, model,
                        input_tokens, output_tokens,
                        cache_creation_tokens, cache_read_tokens,
                        total_cost_usd, provider_id,
                        latency_ms, status_code
                 FROM {}
                 WHERE created_at > ?1
                 ORDER BY created_at ASC
                 LIMIT ?2",
                PROXY_REQUEST_LOGS
            ),
        )?;

        let rows: Vec<CcSwitchLogRow> = stmt
            .query_map(params![last_synced_ts, limit], |row| {
                Ok(CcSwitchLogRow {
                    request_id: row.get::<_, String>(0).unwrap_or_default(),
                    session_id: row.get::<_, Option<String>>(1).unwrap_or(None),
                    created_at: row.get::<_, i64>(2).unwrap_or(0),
                    model: row.get::<_, String>(3).unwrap_or_default(),
                    input_tokens: row.get::<_, i64>(4).unwrap_or(0),
                    output_tokens: row.get::<_, i64>(5).unwrap_or(0),
                    cache_creation_tokens: row.get::<_, i64>(6).unwrap_or(0),
                    cache_read_tokens: row.get::<_, i64>(7).unwrap_or(0),
                    total_cost_usd: parse_cost_text(row.get::<_, String>(8).unwrap_or_default()),
                    provider_id: row.get::<_, String>(9).unwrap_or_default(),
                    latency_ms: row.get::<_, Option<i64>>(10).unwrap_or(None),
                    status_code: row.get::<_, i64>(11).unwrap_or(0),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        // Convert CC Switch rows to TokenOwl UsageRecords
        let records = rows
            .into_iter()
            .map(|row| self.convert_row(row))
            .collect();

        Ok(records)
    }

    /// Get the maximum created_at timestamp currently in the CC Switch database
    pub fn get_max_log_id(&self) -> AppResult<i64> {
        self.open_connection()?;
        let guard = self.conn.lock()
            .map_err(|e| AppError::Config(format!("CC Switch reader mutex poisoned: {}", e)))?;
        let conn = guard.as_ref().ok_or_else(|| {
            AppError::Config("CC Switch connection not available".to_string())
        })?;

        let max_ts: i64 = conn.query_row(
            &format!("SELECT COALESCE(MAX(created_at), 0) FROM {}", PROXY_REQUEST_LOGS),
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        Ok(max_ts)
    }

    /// Convert a CC Switch log row to a TokenOwl UsageRecord
    fn convert_row(&self, row: CcSwitchLogRow) -> UsageRecord {
        // CC Switch stores created_at as unix timestamp in milliseconds
        let timestamp = DateTime::from_timestamp_millis(row.created_at)
            .unwrap_or_else(|| Utc::now());

        let record_id = format!("ccswitch-{}", row.request_id);

        // Calculate total_tokens from components
        let total_tokens = (row.input_tokens + row.output_tokens + 
                           row.cache_creation_tokens + row.cache_read_tokens) as u64;

        UsageRecord {
            id: record_id,
            source: DataSource::CcSwitch,
            session_id: row.session_id.unwrap_or_else(|| row.request_id.clone()),
            timestamp,
            model: row.model,
            tokens: TokenUsage {
                input_tokens: row.input_tokens as u64,
                output_tokens: row.output_tokens as u64,
                cache_creation_tokens: row.cache_creation_tokens as u64,
                cache_read_tokens: row.cache_read_tokens as u64,
                total_tokens,
                reasoning_tokens: 0,  // CC Switch doesn't track reasoning tokens
            },
            cost_usd: row.total_cost_usd,
            project_path: None,  // CC Switch doesn't track project path
            provider_name: Some(row.provider_id),
            response_time_ms: row.latency_ms.map(|v| v as u64),
            status_code: Some(row.status_code as u16),
            cc_switch_log_id: Some(row.created_at.to_string()),
        }
    }
}

/// Parse CC Switch's TEXT cost field to f64
fn parse_cost_text(s: String) -> Option<f64> {
    if s.is_empty() || s == "null" {
        return None;
    }
    s.parse::<f64>().ok()
}


