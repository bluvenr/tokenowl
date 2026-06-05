use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use crate::error::{AppError, AppResult};

/// Raw log entry from CC Switch database (normalized to our internal format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawLogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub app_type: String,
    pub provider_name: Option<String>,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub reasoning_tokens: u64,
    pub total_tokens: u64,
    pub cost: Option<f64>,
    pub status_code: Option<u16>,
    pub response_time_ms: Option<u64>,
}

/// Resolved column names from CC Switch database schema
#[derive(Debug, Clone)]
struct ColumnMapping {
    id_col: String,
    timestamp_col: String,
    app_type_col: String,
    model_col: String,
    input_tokens_col: String,
    output_tokens_col: String,
    provider_col: Option<String>,
    latency_col: Option<String>,
    cache_creation_col: Option<String>,
    cache_read_col: Option<String>,
    reasoning_col: Option<String>,
    total_tokens_col: Option<String>,
    cost_col: Option<String>,
    status_col: Option<String>,
    /// Whether timestamp is stored as Unix milliseconds (integer) vs text
    timestamp_is_unix_ms: bool,
}

/// CC Switch database reader
pub struct CcSwitchReader {
    db_path: PathBuf,
}

impl CcSwitchReader {
    /// Create a new reader with the given database path
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Detect CC Switch installation and return database path
    pub fn detect() -> AppResult<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| AppError::ConfigError("Cannot find home directory".to_string()))?;

        let db_path = home.join(".cc-switch").join("cc-switch.db");

        if db_path.exists() {
            Ok(db_path)
        } else {
            Err(AppError::CcSwitchNotDetected)
        }
    }

    /// Open database in read-only mode
    pub fn open_readonly(&self) -> AppResult<Connection> {
        let conn = Connection::open_with_flags(
            &self.db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        conn.execute_batch("PRAGMA busy_timeout=5000")?;
        Ok(conn)
    }

    /// Probe CC Switch database schema and resolve actual column names
    fn resolve_columns(&self, conn: &Connection) -> AppResult<ColumnMapping> {
        let mut stmt = conn.prepare("PRAGMA table_info(proxy_request_logs)")?;
        let columns: Vec<(String, String)> = stmt
            .query_map([], |row| {
                let name: String = row.get(1)?;
                let col_type: String = row.get(2)?;
                Ok((name, col_type))
            })?
            .filter_map(|r| r.ok())
            .collect();

        if columns.is_empty() {
            return Err(AppError::Database(
                rusqlite::Error::QueryReturnedNoRows.into()
            ));
        }

        let col_names: Vec<&str> = columns.iter().map(|(n, _)| n.as_str()).collect();

        // Resolve ID column: request_id or id
        let id_col = find_column(&col_names, &["request_id", "id"])
            .unwrap_or_else(|| "id".to_string());

        // Resolve timestamp column: created_at or timestamp
        let timestamp_col = find_column(&col_names, &["created_at", "timestamp"])
            .unwrap_or_else(|| "timestamp".to_string());

        // Detect if timestamp is Unix ms (INTEGER type)
        let timestamp_is_unix_ms = columns.iter()
            .find(|(n, _)| n == &timestamp_col)
            .map(|(_, t)| t.to_uppercase().contains("INT"))
            .unwrap_or(false);

        // Resolve app_type column
        let app_type_col = find_column(&col_names, &["app_type", "app", "application"])
            .unwrap_or_else(|| "app_type".to_string());

        // Resolve model column
        let model_col = find_column(&col_names, &["model", "model_name", "model_id"])
            .unwrap_or_else(|| "model".to_string());

        // Resolve token columns
        let input_tokens_col = find_column(&col_names, &["input_tokens", "prompt_tokens"])
            .unwrap_or_else(|| "input_tokens".to_string());
        let output_tokens_col = find_column(&col_names, &["output_tokens", "completion_tokens"])
            .unwrap_or_else(|| "output_tokens".to_string());

        // Optional columns
        let provider_col = find_column(&col_names, &["provider_id", "provider_name", "provider"]);
        let latency_col = find_column(&col_names, &["latency_ms", "response_time_ms", "latency"]);
        let cache_creation_col = find_column(&col_names, &["cache_creation_tokens", "cache_write_tokens"]);
        let cache_read_col = find_column(&col_names, &["cache_read_tokens", "cache_hit_tokens"]);
        let reasoning_col = find_column(&col_names, &["reasoning_tokens"]);
        let total_tokens_col = find_column(&col_names, &["total_tokens"]);
        let cost_col = find_column(&col_names, &["total_cost_usd", "cost_usd", "cost"]);
        let status_col = find_column(&col_names, &["status_code", "status", "http_status"]);

        log::info!(
            "CC Switch schema resolved: id={}, timestamp={} (unix_ms={}), provider={:?}, latency={:?}",
            id_col, timestamp_col, timestamp_is_unix_ms, provider_col, latency_col
        );

        Ok(ColumnMapping {
            id_col,
            timestamp_col,
            app_type_col,
            model_col,
            input_tokens_col,
            output_tokens_col,
            provider_col,
            latency_col,
            cache_creation_col,
            cache_read_col,
            reasoning_col,
            total_tokens_col,
            cost_col,
            status_col,
            timestamp_is_unix_ms,
        })
    }

    /// Read all records since the given timestamp
    pub fn read_since(&self, since: DateTime<Utc>) -> AppResult<Vec<RawLogEntry>> {
        let conn = self.open_readonly()?;
        let mapping = self.resolve_columns(&conn)?;

        // Build SELECT clause with actual column names
        let mut select_cols = vec![
            mapping.id_col.clone(),
            mapping.timestamp_col.clone(),
            mapping.app_type_col.clone(),
            mapping.model_col.clone(),
            mapping.input_tokens_col.clone(),
            mapping.output_tokens_col.clone(),
        ];

        // Track which optional columns are included and their order
        let mut optional_fields: Vec<&str> = Vec::new();

        if let Some(ref col) = mapping.cache_creation_col {
            select_cols.push(col.clone());
            optional_fields.push("cache_creation");
        }
        if let Some(ref col) = mapping.cache_read_col {
            select_cols.push(col.clone());
            optional_fields.push("cache_read");
        }
        if let Some(ref col) = mapping.reasoning_col {
            select_cols.push(col.clone());
            optional_fields.push("reasoning");
        }
        if let Some(ref col) = mapping.total_tokens_col {
            select_cols.push(col.clone());
            optional_fields.push("total_tokens");
        }
        if let Some(ref col) = mapping.provider_col {
            select_cols.push(col.clone());
            optional_fields.push("provider");
        }
        if let Some(ref col) = mapping.cost_col {
            select_cols.push(col.clone());
            optional_fields.push("cost");
        }
        if let Some(ref col) = mapping.status_col {
            select_cols.push(col.clone());
            optional_fields.push("status");
        }
        if let Some(ref col) = mapping.latency_col {
            select_cols.push(col.clone());
            optional_fields.push("latency");
        }

        let query = format!(
            "SELECT {} FROM proxy_request_logs WHERE {} > ?1 ORDER BY {} ASC",
            select_cols.join(", "),
            mapping.timestamp_col,
            mapping.timestamp_col,
        );

        // Convert since to the appropriate format for comparison
        // Use Unix seconds for integer columns (CC Switch uses Unix seconds, not ms)
        let since_param: Box<dyn rusqlite::types::ToSql> = if mapping.timestamp_is_unix_ms {
            Box::new(since.timestamp())
        } else {
            Box::new(since.to_rfc3339())
        };

        let mut stmt = conn.prepare(&query)?;
        let entries = stmt
            .query_map(params![since_param.as_ref()], |row| {
                let mut idx = 0;

                // Core columns (always present)
                let id: String = row.get(idx)?; idx += 1;

                // Parse timestamp based on storage format
                let timestamp = if mapping.timestamp_is_unix_ms {
                    let raw: i64 = row.get(idx)?; idx += 1;
                    // Auto-detect: values > 1e12 are milliseconds, otherwise seconds
                    let (secs, nsecs) = if raw > 1_000_000_000_000 {
                        (raw / 1000, ((raw % 1000) * 1_000_000) as u32)
                    } else {
                        (raw, 0u32)
                    };
                    DateTime::from_timestamp(secs, nsecs)
                        .unwrap_or_else(|| Utc::now())
                } else {
                    let ts_str: String = row.get(idx)?; idx += 1;
                    DateTime::parse_from_rfc3339(&ts_str)
                        .unwrap_or_else(|_| Utc::now().into())
                        .with_timezone(&Utc)
                };

                let app_type: String = row.get(idx)?; idx += 1;
                let model: String = row.get(idx)?; idx += 1;
                let input_tokens: u64 = row.get(idx)?; idx += 1;
                let output_tokens: u64 = row.get(idx)?; idx += 1;

                // Optional columns in the order they were added
                let mut cache_creation_tokens: u64 = 0;
                let mut cache_read_tokens: u64 = 0;
                let mut reasoning_tokens: u64 = 0;
                let mut total_tokens: u64 = 0;
                let mut has_total_tokens = false;
                let mut provider_name: Option<String> = None;
                let mut cost: Option<f64> = None;
                let mut status_code: Option<u16> = None;
                let mut response_time_ms: Option<u64> = None;

                for field in &optional_fields {
                    match *field {
                        "cache_creation" => {
                            cache_creation_tokens = row.get::<_, Option<u64>>(idx)?.unwrap_or(0);
                            idx += 1;
                        }
                        "cache_read" => {
                            cache_read_tokens = row.get::<_, Option<u64>>(idx)?.unwrap_or(0);
                            idx += 1;
                        }
                        "reasoning" => {
                            reasoning_tokens = row.get::<_, Option<u64>>(idx)?.unwrap_or(0);
                            idx += 1;
                        }
                        "total_tokens" => {
                            total_tokens = row.get::<_, Option<u64>>(idx)?.unwrap_or(0);
                            has_total_tokens = true;
                            idx += 1;
                        }
                        "provider" => {
                            provider_name = row.get::<_, Option<String>>(idx)?;
                            idx += 1;
                        }
                        "cost" => {
                            // CC Switch may store cost as TEXT, try both
                            cost = row.get::<_, Option<f64>>(idx).ok().flatten()
                                .or_else(|| {
                                    row.get::<_, Option<String>>(idx).ok().flatten()
                                        .and_then(|s| s.parse::<f64>().ok())
                                });
                            idx += 1;
                        }
                        "status" => {
                            status_code = row.get::<_, Option<u16>>(idx)?;
                            idx += 1;
                        }
                        "latency" => {
                            response_time_ms = row.get::<_, Option<u64>>(idx)?;
                            idx += 1;
                        }
                        _ => {}
                    }
                }

                // If total_tokens not in schema, calculate from components
                if !has_total_tokens {
                    total_tokens = input_tokens + output_tokens
                        + cache_creation_tokens + cache_read_tokens;
                }

                Ok(RawLogEntry {
                    id,
                    timestamp,
                    app_type,
                    provider_name,
                    model,
                    input_tokens,
                    output_tokens,
                    cache_creation_tokens,
                    cache_read_tokens,
                    reasoning_tokens,
                    total_tokens,
                    cost,
                    status_code,
                    response_time_ms,
                })
            })?
            .filter_map(|r| {
                match r {
                    Ok(entry) => Some(entry),
                    Err(e) => {
                        log::warn!("Skipping malformed CC Switch row: {}", e);
                        None
                    }
                }
            })
            .collect();

        Ok(entries)
    }

    /// Get total record count
    pub fn total_count(&self) -> AppResult<u64> {
        let conn = self.open_readonly()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM proxy_request_logs",
            [],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    /// Check if CC Switch proxy is running (check for listening process)
    pub fn is_proxy_running(&self) -> bool {
        // Try to check if CC Switch process is running
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            // Check if port 8080 is listening (CC Switch default port)
            if let Ok(output) = Command::new("netstat").args(["-an"]).output() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    // CC Switch typically listens on 8080
                    if text.contains("LISTENING") && text.contains(":8080") {
                        return true;
                    }
                }
            }
            // Fallback: check for cc-switch process
            if let Ok(output) = Command::new("tasklist")
                .args(["/FI", "IMAGENAME eq cc-switch.exe", "/NH"])
                .output()
            {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    return text.contains("cc-switch.exe");
                }
            }
            false
        }
        #[cfg(not(target_os = "windows"))]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("pgrep").arg("-f").arg("cc-switch").output() {
                return output.status.success();
            }
            false
        }
    }
}

/// Find the first matching column name from a list of candidates
fn find_column(columns: &[&str], candidates: &[&str]) -> Option<String> {
    for candidate in candidates {
        if columns.iter().any(|c| c.eq_ignore_ascii_case(candidate)) {
            return Some(candidate.to_string());
        }
    }
    None
}
