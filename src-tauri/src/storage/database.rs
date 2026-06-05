use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::AppResult;
use super::migrations;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new() -> AppResult<Self> {
        let db_path = Self::db_path()?;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        // Run integrity check on startup
        let integrity: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap_or_else(|_| "error".to_string());
        if integrity != "ok" {
            log::error!("Database integrity check failed: {}", integrity);
        }

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn db_path() -> AppResult<PathBuf> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| crate::error::AppError::Config("Cannot get data directory".into()))?;
        Ok(data_dir.join(crate::APP_DATA_DIR).join("tokenowl.db"))
    }

    fn run_migrations(&self) -> AppResult<()> {
        let conn = self.conn.lock()
            .map_err(|e| crate::error::AppError::Config(format!("DB mutex poisoned: {}", e)))?;
        migrations::run_all(&conn)?;
        Ok(())
    }

    pub fn conn(&self) -> AppResult<std::sync::MutexGuard<'_, Connection>> {
        self.conn.lock()
            .map_err(|e| crate::error::AppError::Config(format!("DB mutex poisoned: {}", e)))
    }

    /// Get the database file size in bytes
    pub fn get_db_file_size(&self) -> Option<u64> {
        Self::db_path().ok().and_then(|p| std::fs::metadata(p).ok()).map(|m| m.len())
    }
}
