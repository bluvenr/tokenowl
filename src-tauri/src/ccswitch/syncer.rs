use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::error::AppResult;
use crate::storage::database::Database;
use super::reader::{CcSwitchReader, RawLogEntry};

/// Sync result statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub new_records: u64,
    pub skipped_duplicates: u64,
    pub errors: u64,
    pub sync_duration_ms: u64,
}

/// CC Switch data syncer
pub struct CcSwitchSyncer {
    reader: Option<CcSwitchReader>,
    db: Arc<Database>,
}

impl CcSwitchSyncer {
    /// Create a new syncer
    pub fn new(db: Arc<Database>) -> Self {
        let reader = CcSwitchReader::detect().ok().map(CcSwitchReader::new);
        Self { reader, db }
    }

    /// Check if CC Switch is detected
    pub fn is_detected(&self) -> bool {
        self.reader.is_some()
    }

    /// Get the CC Switch database path
    pub fn db_path(&self) -> Option<std::path::PathBuf> {
        CcSwitchReader::detect().ok()
    }

    /// Get the last sync time from the database
    pub fn last_sync_time(&self) -> Option<String> {
        self.get_last_sync_time().ok().flatten().map(|dt| dt.to_rfc3339())
    }

    /// Perform incremental sync
    pub fn sync(&mut self) -> AppResult<SyncResult> {
        let start = Instant::now();

        // Re-detect CC Switch if not previously detected
        if self.reader.is_none() {
            if let Ok(path) = CcSwitchReader::detect() {
                log::info!("CC Switch detected at {:?}", path);
                self.reader = Some(CcSwitchReader::new(path));
            }
        }

        let reader = match &self.reader {
            Some(r) => r,
            None => {
                return Ok(SyncResult {
                    new_records: 0,
                    skipped_duplicates: 0,
                    errors: 0,
                    sync_duration_ms: start.elapsed().as_millis() as u64,
                });
            }
        };

        // Get last sync time from database
        let last_sync = self.get_last_sync_time()?;
        let since = last_sync.unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());

        // Read new records from CC Switch (before acquiring TokenOwl DB lock)
        let raw_entries = reader.read_since(since)?;

        log::info!("Read {} entries from CC Switch, starting DB insert", raw_entries.len());

        // Acquire DB connection once and hold for the entire transaction
        let mut conn = self.db.conn()?;

        // Use rusqlite transaction for automatic rollback on failure
        let result = {
            let tx = conn.transaction()?;

            let mut new_records = 0u64;
            let mut skipped = 0u64;
            let mut errors = 0u64;

            for entry in &raw_entries {
                match insert_record(&tx, entry) {
                    Ok(inserted) => {
                        if inserted {
                            new_records += 1;
                        } else {
                            skipped += 1;
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to insert CC Switch record {}: {}", entry.id, e);
                        errors += 1;
                    }
                }
            }

            // Update sync state within the same transaction
            let now = Utc::now().to_rfc3339();
            tx.execute(
                "UPDATE sync_state SET
                    last_sync_time = ?1,
                    last_sync_record_count = ?2,
                    total_records_synced = total_records_synced + ?2,
                    cc_switch_detected = 1
                WHERE id = 1",
                rusqlite::params![now, new_records],
            )?;

            tx.commit()?;

            Ok(SyncResult {
                new_records,
                skipped_duplicates: skipped,
                errors,
                sync_duration_ms: start.elapsed().as_millis() as u64,
            })
        };

        // If result is Err, the Transaction is dropped here, which automatically rolls back
        result
    }

    /// Get the last sync time from database
    fn get_last_sync_time(&self) -> AppResult<Option<DateTime<Utc>>> {
        let conn = self.db.conn()?;
        let result: Option<String> = conn.query_row(
            "SELECT last_sync_time FROM sync_state WHERE id = 1",
            [],
            |row| row.get(0),
        )?;

        Ok(result.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))))
    }
}

/// Insert a single record into the database
fn insert_record(conn: &rusqlite::Connection, entry: &RawLogEntry) -> AppResult<bool> {
    // Generate deterministic UUID for deduplication
    let namespace = Uuid::NAMESPACE_DNS;
    let unique_key = format!("{}:{}:{}", entry.app_type, entry.id, entry.timestamp);
    let id = Uuid::new_v5(&namespace, unique_key.as_bytes()).to_string();

    // Check if record already exists
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM usage_records WHERE id = ?1)",
        [id.clone()],
        |row| row.get(0),
    )?;

    if exists {
        return Ok(false);
    }

    // Insert new record
    conn.execute(
        "INSERT INTO usage_records (
            id, timestamp, app_type, provider_name, model,
            input_tokens, output_tokens, cache_creation_tokens, cache_read_tokens,
            reasoning_tokens, total_tokens, cost_usd, status_code, response_time_ms,
            cc_switch_log_id
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        rusqlite::params![
            id,
            entry.timestamp.to_rfc3339(),
            entry.app_type,
            entry.provider_name,
            entry.model,
            entry.input_tokens,
            entry.output_tokens,
            entry.cache_creation_tokens,
            entry.cache_read_tokens,
            entry.reasoning_tokens,
            entry.total_tokens,
            entry.cost,
            entry.status_code,
            entry.response_time_ms,
            entry.id,
        ],
    )?;

    Ok(true)
}

// ── Background auto-sync ─────────────────────────────────────────────

/// Shared syncer state (same type used in Tauri commands)
pub type CcSwitchSyncerState = Arc<Mutex<CcSwitchSyncer>>;

/// Handle for the background sync task.
/// When dropped, the background task is automatically aborted.
pub struct SyncHandle {
    #[allow(dead_code)]
    thread_handle: std::thread::JoinHandle<()>,
}

impl SyncHandle {
    /// Abort the background sync task.
    pub fn stop(&self) {
        // Note: std::thread doesn't have abort, thread will stop when process exits
    }
}

impl Drop for SyncHandle {
    fn drop(&mut self) {
        // Thread will be cleaned up when process exits
    }
}

/// Start background periodic sync of CC Switch data.
///
/// - Syncs immediately on startup (after 5s delay for app initialization)
/// - Reads interval from `sync_state.sync_interval_secs` each cycle (dynamic)
/// - Re-detects CC Switch on each cycle if not previously found
/// - Skips sync when interval is 0 (disabled)
pub fn start_background_sync(
    syncer: CcSwitchSyncerState,
    db: Arc<Database>,
) -> SyncHandle {
    log::info!("Starting background sync");

    let handle = std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            sync_loop(syncer, db);
        }));
        if let Err(e) = result {
            log::error!("Background sync thread panicked: {:?}", e);
        }
    });

    SyncHandle {
        thread_handle: handle,
    }
}

fn sync_loop(syncer: CcSwitchSyncerState, db: Arc<Database>) {
    // Short initial delay to let the app settle, then sync immediately
    std::thread::sleep(std::time::Duration::from_secs(5));

    loop {
        // Read interval dynamically from database each cycle
        let interval_secs = get_sync_interval_secs(&db);

        // Skip this cycle if sync is disabled (interval = 0)
        if interval_secs > 0 {
            log::debug!("Background sync tick (interval: {}s)", interval_secs);

            match syncer.lock() {
                Ok(mut s) => {
                    match s.sync() {
                        Ok(result) => {
                            log::info!(
                                "Background sync: {} new, {} skipped, {} errors ({}ms)",
                                result.new_records,
                                result.skipped_duplicates,
                                result.errors,
                                result.sync_duration_ms,
                            );
                        }
                        Err(e) => {
                            log::warn!("Background sync failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Background sync lock error: {}", e);
                }
            }
        }

        // Sleep until next cycle (use 60s minimum to avoid busy loop when disabled)
        let sleep_secs = if interval_secs == 0 { 60 } else { interval_secs };
        std::thread::sleep(std::time::Duration::from_secs(sleep_secs));
    }
}

/// Read the configured sync interval from the database (seconds).
/// Falls back to 300 (5 minutes) on any error.
pub fn get_sync_interval_secs(db: &Arc<Database>) -> u64 {
    db.conn()
        .ok()
        .and_then(|conn| {
            conn.query_row(
                "SELECT sync_interval_secs FROM sync_state WHERE id = 1",
                [],
                |row| row.get::<_, u64>(0),
            )
            .ok()
        })
        .unwrap_or(300)
}
