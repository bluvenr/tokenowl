use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use crate::storage::database::Database;
use crate::error::AppResult;

/// Crash logger - stores crash information in the database
pub struct CrashLogger {
    db: Arc<Database>,
}

impl CrashLogger {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Log a crash/error to the database
    pub fn log(&self, error_type: &str, message: &str, backtrace: Option<&str>) -> AppResult<()> {
        let conn = self.db.conn()?;
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO crash_logs (id, timestamp, error_type, message, backtrace)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, timestamp, error_type, message, backtrace],
        )?;

        log::error!("Crash logged: {} - {}", error_type, message);
        Ok(())
    }

    /// Get recent crash logs
    pub fn get_recent(&self, limit: u32) -> AppResult<Vec<CrashLogEntry>> {
        let conn = self.db.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, error_type, message, backtrace
            FROM crash_logs
            ORDER BY timestamp DESC
            LIMIT ?1"
        )?;

        let entries: Vec<CrashLogEntry> = stmt
            .query_map([limit], |row| {
                Ok(CrashLogEntry {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    error_type: row.get(2)?,
                    message: row.get(3)?,
                    backtrace: row.get(4)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }

    /// Delete a specific crash log
    pub fn delete(&self, id: &str) -> AppResult<()> {
        let conn = self.db.conn()?;
        conn.execute("DELETE FROM crash_logs WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Clear all crash logs
    pub fn clear_all(&self) -> AppResult<()> {
        let conn = self.db.conn()?;
        conn.execute("DELETE FROM crash_logs", [])?;
        Ok(())
    }
}

/// Crash log entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrashLogEntry {
    pub id: String,
    pub timestamp: String,
    pub error_type: String,
    pub message: String,
    pub backtrace: Option<String>,
}
