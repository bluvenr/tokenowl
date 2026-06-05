use std::sync::Arc;
use std::time::Instant;

use crate::ccswitch::reader::CcSwitchReader;
use crate::error::AppResult;
use crate::models::usage::{CcSwitchStatus, SyncResult, SyncState};
use crate::storage::database::Database;

/// Sync engine: reads new records from CC Switch and inserts them into TokenOwl's database
pub struct CcSwitchSyncer {
    reader: CcSwitchReader,
    db: Arc<Database>,
}

impl CcSwitchSyncer {
    pub fn new(db: Arc<Database>) -> Self {
        let reader = CcSwitchReader::new();
        Self { reader, db }
    }

    /// Create with a custom CC Switch database path
    pub fn with_path(db: Arc<Database>, path: std::path::PathBuf) -> Self {
        let reader = CcSwitchReader::with_path(path);
        Self { reader, db }
    }

    /// Check if CC Switch is detected (database file exists)
    pub fn is_detected(&self) -> bool {
        self.reader.detect()
    }

    /// Get combined CC Switch status: detection + sync state
    pub fn get_status(&self) -> AppResult<CcSwitchStatus> {
        let sync_state = self.db.get_sync_state()?;
        let detected = self.reader.detect();

        if !detected {
            return Ok(CcSwitchStatus {
                detected: false,
                db_path: None,
                db_size_bytes: None,
                record_count: None,
                is_running: false,
                sync_state,
            });
        }

        let info = self.reader.get_info().ok();

        Ok(CcSwitchStatus {
            detected: true,
            db_path: info.as_ref().map(|i| i.db_path.clone()),
            db_size_bytes: info.as_ref().map(|i| i.db_size_bytes),
            record_count: info.as_ref().map(|i| i.record_count),
            is_running: detected && info.as_ref().map(|i| i.record_count > 0).unwrap_or(false),
            sync_state,
        })
    }

    /// Get the CC Switch database path
    pub fn db_path(&self) -> &std::path::PathBuf {
        self.reader.db_path()
    }

    /// Run a sync cycle: read new records from CC Switch and insert into TokenOwl
    pub fn sync(&self) -> AppResult<SyncResult> {
        let start_time = Instant::now();

        // Check if CC Switch is available
        if !self.reader.detect() {
            // Update sync state to mark CC Switch as not detected
            let mut state = self.db.get_sync_state()?;
            state.cc_switch_detected = false;
            self.db.update_sync_state(&state)?;
            return Ok(SyncResult {
                new_records: 0,
                skipped_duplicates: 0,
                errors: 0,
                sync_duration_ms: start_time.elapsed().as_millis() as u64,
            });
        }

        // Get the last synced timestamp from our database
        let _sync_state = self.db.get_sync_state()?;
        // Use the last sync time as the cursor (or 0 for first sync)
        let last_synced_ts: i64 = _sync_state.last_sync_time
            .as_ref()
            .and_then(|t| {
                // Try parsing as RFC3339 first, then fallback to 0
                chrono::DateTime::parse_from_rfc3339(t)
                    .ok()
                    .map(|dt| dt.timestamp_millis())
            })
            .unwrap_or(0);

        // Read new records in batches using timestamp cursor
        let batch_size = 500u32;
        let mut total_new = 0u64;
        let mut total_skipped = 0u64;
        let mut total_errors = 0u64;
        let mut current_max_ts = last_synced_ts;

        loop {
            let records = match self.reader.read_new_records(current_max_ts, batch_size) {
                Ok(r) => r,
                Err(e) => {
                    log::error!("Failed to read CC Switch records: {}", e);
                    total_errors += 1;
                    break;
                }
            };

            if records.is_empty() {
                break;
            }

            // Track the max timestamp we've seen
            if let Some(last) = records.last() {
                let ts_ms = last.timestamp.timestamp_millis();
                if ts_ms > current_max_ts {
                    current_max_ts = ts_ms;
                }
            }

            let batch_count = records.len() as u64;

            // Insert records (duplicates are skipped automatically)
            match self.db.insert_records(&records) {
                Ok(inserted) => {
                    total_new += inserted as u64;
                    total_skipped += batch_count - inserted as u64;
                }
                Err(e) => {
                    log::error!("Failed to insert CC Switch records: {}", e);
                    total_errors += 1;
                }
            }

            // If we got fewer than batch_size, we've reached the end
            if (batch_count as u32) < batch_size {
                break;
            }
        }

        // Update sync state
        let total_records = self.db.get_total_record_count().unwrap_or(0);
        let now = chrono::Utc::now().to_rfc3339();
        let new_state = SyncState {
            last_sync_time: Some(now),
            last_sync_record_count: total_new,
            cc_switch_db_path: self.reader.db_path().to_string_lossy().to_string(),
            cc_switch_detected: true,
            sync_interval_secs: _sync_state.sync_interval_secs,
            total_records_synced: total_records,
        };
        self.db.update_sync_state(&new_state)?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        if total_new > 0 {
            log::info!(
                "CC Switch sync: {} new records, {} duplicates skipped, {} errors ({}ms)",
                total_new, total_skipped, total_errors, duration_ms
            );
        }

        Ok(SyncResult {
            new_records: total_new,
            skipped_duplicates: total_skipped,
            errors: total_errors,
            sync_duration_ms: duration_ms,
        })
    }
}
