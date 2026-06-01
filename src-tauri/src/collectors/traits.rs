use crate::error::AppResult;
use crate::models::usage::{DataSource, UsageRecord};
use std::path::PathBuf;

/// All data collectors must implement this trait
pub trait Collector: Send + Sync {
    /// Data source identifier
    fn source(&self) -> DataSource;

    /// Absolute paths to watch for file changes
    fn watch_paths(&self) -> Vec<PathBuf>;

    /// Full scan: parse all existing data on first launch
    fn full_scan(&self) -> AppResult<Vec<UsageRecord>>;

    /// Incremental parse: extract new records from a changed file
    /// Returns (new records, new byte offset)
    fn incremental_parse(
        &self,
        file_path: &PathBuf,
        from_offset: u64,
    ) -> AppResult<(Vec<UsageRecord>, u64)>;

    /// Check if this data source is available on the current system
    fn is_available(&self) -> bool;

    /// List all scannable data files (for recording offsets after full scan)
    fn scanned_files(&self) -> AppResult<Vec<PathBuf>>;

    /// File extensions this collector processes
    fn file_extensions(&self) -> &[&str];

    /// Whether this collector uses JSON-style whole-file parsing
    /// (true = JSON files that get rewritten; false = JSONL append-only)
    fn is_whole_file(&self) -> bool { false }

    /// Derive a session_id from a file path for use in process_file_change.
    /// Default: use file stem. Override for collectors with different session_id logic.
    fn session_id_for_file(&self, path: &std::path::Path) -> String {
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default()
    }
}
