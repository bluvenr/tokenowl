use rusqlite::Connection;
use std::sync::Mutex;
use crate::error::{AppError, AppResult};
use crate::storage::migrations;

/// SQLite database wrapper
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Create a new database instance
    pub fn new() -> AppResult<Self> {
        let db_path = Self::get_db_path()?;

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Self::open_with_integrity(&db_path)?;

        // Set pragmas for safety and performance
        conn.execute_batch("PRAGMA journal_mode=WAL")?;
        conn.execute_batch("PRAGMA synchronous=NORMAL")?;
        conn.execute_batch("PRAGMA foreign_keys=ON")?;
        // Limit cache size to 8MB to prevent excessive memory usage
        conn.execute_batch("PRAGMA cache_size=-8192")?;
        // Limit mmap to 256MB
        conn.execute_batch("PRAGMA mmap_size=268435456")?;

        let db = Self {
            conn: Mutex::new(conn),
        };

        // Run migrations
        {
            let conn = db.conn.lock().map_err(|_| AppError::Database(rusqlite::Error::InvalidParameterName("lock".to_string())))?;
            migrations::run_migrations(&conn)?;
        }

        Ok(db)
    }

    /// Open database and verify integrity. If corrupted, backup and recreate.
    fn open_with_integrity(db_path: &std::path::Path) -> AppResult<Connection> {
        let conn = Connection::open(db_path)?;

        // Quick integrity check
        let integrity_ok: bool = conn
            .query_row("PRAGMA quick_check", [], |row| {
                let result: String = row.get(0)?;
                Ok(result == "ok")
            })
            .unwrap_or(false);

        if !integrity_ok {
            log::warn!("Database integrity check failed, backing up and recreating");
            drop(conn);

            // Backup the corrupted database
            let backup_path = db_path.with_extension("db.corrupted");
            let _ = std::fs::rename(db_path, &backup_path);
            // Also remove WAL and SHM files
            let wal_path = db_path.with_extension("db-wal");
            let shm_path = db_path.with_extension("db-shm");
            let _ = std::fs::remove_file(&wal_path);
            let _ = std::fs::remove_file(&shm_path);

            // Open fresh database
            Ok(Connection::open(db_path)?)
        } else {
            Ok(conn)
        }
    }

    /// Get database file path
    fn get_db_path() -> AppResult<std::path::PathBuf> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| AppError::ConfigError("Cannot find data directory".to_string()))?;
        Ok(data_dir.join(crate::APP_DATA_DIR).join("tokenowl.db"))
    }

    /// Get a reference to the database connection
    pub fn conn(&self) -> AppResult<std::sync::MutexGuard<'_, Connection>> {
        self.conn.lock().map_err(|_| AppError::Database(rusqlite::Error::InvalidParameterName("lock".to_string())))
    }
}
