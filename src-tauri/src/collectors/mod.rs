pub mod traits;
pub mod claude_code;
pub mod codex_cli;
pub mod gemini_cli;
pub mod kimi_code;
pub mod qwen_code;

use crate::error::AppResult;
use crate::models::usage::{DataSource, UsageRecord};
use crate::models::settings::ModelPricing;
use crate::pricing::calculator::calculate_cost;
use crate::pricing::registry::{load_cached_prices, merge_prices};
use crate::storage::database::Database;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use claude_code::ClaudeCodeCollector;
use codex_cli::CodexCliCollector;
use gemini_cli::GeminiCliCollector;
use kimi_code::KimiCodeCollector;
use qwen_code::QwenCodeCollector;
use traits::Collector;

/// Namespace UUID for TokenOwl record IDs (UUID v5)
const TOKENOWL_NS: uuid::Uuid = uuid::Uuid::from_bytes([
    0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1,
    0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
]);

/// Generate a deterministic record ID from key fields.
/// Same input always produces the same UUID, enabling dedup on rescan.
pub fn make_record_id(
    source: &DataSource,
    session_id: &str,
    timestamp_rfc3339: &str,
    model: &str,
    discriminator: &str,
) -> String {
    let key = format!(
        "{}:{}:{}:{}:{}",
        source.as_str(),
        session_id,
        timestamp_rfc3339,
        model,
        discriminator,
    );
    uuid::Uuid::new_v5(&TOKENOWL_NS, key.as_bytes()).to_string()
}

/// Status of a data source
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceStatus {
    pub source: String,
    pub display_name: String,
    pub available: bool,
    pub enabled: bool,
    pub record_count: u64,
    pub last_error: Option<String>,
}

/// Manages all collectors: lifecycle, full scan, incremental updates
pub struct CollectorManager {
    collectors: Vec<Box<dyn Collector>>,
    db: Arc<Database>,
}

impl CollectorManager {
    pub fn new(db: Arc<Database>) -> Self {
        let collectors: Vec<Box<dyn Collector>> = vec![
            Box::new(ClaudeCodeCollector::new()),
            Box::new(CodexCliCollector::new()),
            Box::new(GeminiCliCollector::new()),
            Box::new(KimiCodeCollector::new()),
            Box::new(QwenCodeCollector::new()),
        ];
        Self { collectors, db }
    }

    /// Run full scan on all available collectors, record file offsets, and backfill costs
    pub fn initial_scan(&self) -> AppResult<()> {
        log::info!("Starting initial scan across all collectors...");
        let mut total_inserted = 0u64;

        for collector in &self.collectors {
            let source = collector.source();
            let name = source.display_name();

            if !collector.is_available() {
                log::info!("[{}] Not available, skipping", name);
                continue;
            }

            // Check if source is enabled in config
            if !self.is_source_enabled(source.as_str()) {
                log::info!("[{}] Disabled by user config, skipping", name);
                continue;
            }

            log::info!("[{}] Running full scan...", name);
            match collector.full_scan() {
                Ok(records) => {
                    let count = self.db.insert_records(&records)?;
                    total_inserted += count as u64;
                    log::info!("[{}] Inserted {} new records", name, count);

                    // Record file offsets for incremental parsing on restart
                    self.record_all_file_offsets(collector.as_ref())?;
                }
                Err(e) => {
                    log::error!("[{}] Full scan failed: {}", name, e);
                }
            }
        }

        // Backfill costs for records without cost_usd
        let backfilled = self.backfill_costs()?;
        log::info!(
            "Initial scan complete: {} records inserted, {} costs backfilled",
            total_inserted, backfilled
        );

        Ok(())
    }

    /// Process a file change event from the watcher
    pub fn process_file_change(&self, path: &std::path::Path) -> AppResult<bool> {
        let path_buf = path.to_path_buf();

        // Find matching collector
        let collector = self.find_collector_for_path(&path_buf);
        let collector = match collector {
            Some(c) => c,
            None => return Ok(false),
        };

        let source = collector.source();

        // Check if source is enabled
        if !self.is_source_enabled(source.as_str()) {
            return Ok(false);
        }

        let file_path_str = path.to_string_lossy().to_string();
        log::debug!("[{}] Processing file change: {}", source.display_name(), file_path_str);

        if collector.is_whole_file() {
            // JSON files: re-parse entirely on change
            let session_id = collector.session_id_for_file(&path_buf);

            // Parse FIRST — if it fails, keep old records intact
            let parse_result = collector.incremental_parse(&path_buf, 0);
            match parse_result {
                Ok((records, new_offset)) => {
                    // Parse succeeded: atomically replace old records with new
                    let count = self.db.replace_session_records(source.as_str(), &session_id, &records)?;
                    self.db.set_file_offset(&file_path_str, source.as_str(), new_offset)?;

                    if count > 0 {
                        self.backfill_costs_for_records(&records)?;
                        log::info!("[{}] Re-parsed {}: {} new records", source.display_name(), session_id, count);
                        return Ok(true);
                    }
                }
                Err(e) => {
                    // Parse failed: log warning, keep existing records
                    log::warn!(
                        "[{}] Re-parse failed for {:?}, keeping existing records: {}",
                        source.display_name(), path, e
                    );
                }
            }
        } else {
            // JSONL files: incremental parse from last offset
            let offset = self.db.get_file_offset(&file_path_str)?;
            let (records, new_offset) = collector.incremental_parse(&path_buf, offset)?;

            if !records.is_empty() {
                let count = self.db.insert_records(&records)?;
                self.db.set_file_offset(&file_path_str, source.as_str(), new_offset)?;

                if count > 0 {
                    self.backfill_costs_for_records(&records)?;
                    log::info!(
                        "[{}] Incremental: {} new records from {}",
                        source.display_name(),
                        count,
                        file_path_str
                    );
                    return Ok(true);
                }
            } else {
                // Still update offset even if no new records
                self.db.set_file_offset(&file_path_str, source.as_str(), new_offset)?;
            }
        }

        Ok(false)
    }

    /// Get status of all data sources
    pub fn get_source_status(&self) -> Vec<SourceStatus> {
        self.collectors
            .iter()
            .map(|c| {
                let source = c.source();
                let available = c.is_available();
                let enabled = self.is_source_enabled(source.as_str());
                let record_count = self.db.count_source_records(source.as_str()).unwrap_or(0);
                SourceStatus {
                    source: source.as_str().to_string(),
                    display_name: source.display_name().to_string(),
                    available,
                    enabled,
                    record_count,
                    last_error: None,
                }
            })
            .collect()
    }

    /// Get all watch paths from available collectors
    pub fn all_watch_paths(&self) -> Vec<PathBuf> {
        self.collectors
            .iter()
            .filter(|c| c.is_available() && self.is_source_enabled(c.source().as_str()))
            .flat_map(|c| c.watch_paths())
            .collect()
    }

    /// Run a full rescan (triggered manually from frontend)
    pub fn rescan(&self) -> AppResult<u64> {
        let mut total = 0u64;
        for collector in &self.collectors {
            if !collector.is_available() || !self.is_source_enabled(collector.source().as_str()) {
                continue;
            }
            match collector.full_scan() {
                Ok(records) => {
                    total += self.db.insert_records(&records)? as u64;
                    self.record_all_file_offsets(collector.as_ref())?;
                }
                Err(e) => {
                    log::error!("[{}] Rescan failed: {}", collector.source().display_name(), e);
                }
            }
        }
        self.backfill_costs()?;
        Ok(total)
    }

    /// Recalculate costs for a specific model: nullify existing costs then backfill
    /// with the current (updated) price. Returns the number of affected records.
    pub fn recalculate_model_costs(&self, model_id: &str) -> AppResult<u64> {
        let count = self.db.invalidate_costs_for_model(model_id)?;
        if count > 0 {
            self.backfill_costs()?;
            log::info!("Recalculated costs for {} records of model '{}'", count, model_id);
        }
        Ok(count)
    }

    /// Check if a path belongs to any collector's file extensions
    pub fn matches_any_collector(&self, path: &std::path::Path) -> bool {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        self.collectors.iter().any(|c| {
            c.is_available()
                && self.is_source_enabled(c.source().as_str())
                && c.file_extensions().contains(&ext)
        })
    }

    // ─── Internal helpers ────────────────────────────────────────────

    fn find_collector_for_path(&self, path: &PathBuf) -> Option<&dyn Collector> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        for collector in &self.collectors {
            if !collector.is_available() {
                continue;
            }
            if !collector.file_extensions().contains(&ext) {
                continue;
            }

            // Special case for Kimi: only context.jsonl files
            if collector.source() == DataSource::KimiCode && file_name != "context.jsonl" {
                continue;
            }

            // Check if path is under one of the collector's watch paths
            for watch_path in collector.watch_paths() {
                if path.starts_with(&watch_path) {
                    return Some(collector.as_ref());
                }
            }
        }
        None
    }

    fn record_all_file_offsets(&self, collector: &dyn Collector) -> AppResult<()> {
        let source = collector.source();
        match collector.scanned_files() {
            Ok(files) => {
                let file_count = files.len();
                for file_path in files {
                    let offset = std::fs::metadata(&file_path)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    self.db.set_file_offset(
                        &file_path.to_string_lossy(),
                        source.as_str(),
                        offset,
                    )?;
                }
                log::debug!(
                    "[{}] Recorded offsets for {} files",
                    source.display_name(),
                    file_count
                );
            }
            Err(e) => {
                log::warn!(
                    "[{}] Failed to list scanned files: {}",
                    source.display_name(),
                    e
                );
            }
        }
        Ok(())
    }

    fn is_source_enabled(&self, source: &str) -> bool {
        match self.db.get_source_configs() {
            Ok(configs) => configs
                .iter()
                .find(|c| c.source == source)
                .map(|c| c.enabled)
                .unwrap_or(true), // default to enabled if not configured
            Err(_) => true,
        }
    }

    /// Build a merged price map (cached + remote + custom) for cost calculation
    fn build_price_map_with_remote(&self, remote: &[ModelPricing]) -> HashMap<String, ModelPricing> {
        let cached = load_cached_prices();
        let custom = self.db.get_custom_prices().unwrap_or_default();
        let merged = merge_prices(&cached, remote, &custom);
        merged.into_iter().map(|p| (p.model_id.clone(), p)).collect()
    }

    /// Build a merged price map (cached + custom) for cost calculation
    fn build_price_map(&self) -> HashMap<String, ModelPricing> {
        self.build_price_map_with_remote(&[])
    }

    /// Backfill costs using remote prices (called after remote price sync)
    pub fn backfill_costs_with_remote(&self, remote_prices: &[ModelPricing]) -> AppResult<u64> {
        let price_map = self.build_price_map_with_remote(remote_prices);
        let mut total_count = 0u64;
        const BATCH_SIZE: u32 = 1000;

        loop {
            let records = self.db.get_records_without_cost(BATCH_SIZE)?;
            if records.is_empty() {
                break;
            }

            let mut batch_updated = 0u64;
            for record in &records {
                if let Some(price) = Self::find_price(&price_map, &record.model, &record.source) {
                    let cost = calculate_cost(&record.tokens, price);
                    self.db.update_record_cost(&record.id, cost)?;
                    total_count += 1;
                    batch_updated += 1;
                }
            }

            if records.len() < BATCH_SIZE as usize {
                break;
            }
            // If no records in this batch could be resolved, break to avoid infinite loop
            if batch_updated == 0 {
                log::warn!(
                    "backfill_costs_with_remote: {} records have no matching price, skipping",
                    records.len()
                );
                break;
            }
        }

        if total_count > 0 {
            log::info!("Backfilled cost for {} records with remote prices", total_count);
        }
        Ok(total_count)
    }

    /// Backfill cost_usd for all records that don't have one
    fn backfill_costs(&self) -> AppResult<u64> {
        let price_map = self.build_price_map();
        let mut total_count = 0u64;
        const BATCH_SIZE: u32 = 1000;

        loop {
            let records = self.db.get_records_without_cost(BATCH_SIZE)?;
            if records.is_empty() {
                break;
            }

            let mut batch_updated = 0u64;
            for record in &records {
                if let Some(price) = Self::find_price(&price_map, &record.model, &record.source) {
                    let cost = calculate_cost(&record.tokens, price);
                    self.db.update_record_cost(&record.id, cost)?;
                    total_count += 1;
                    batch_updated += 1;
                }
            }

            if records.len() < BATCH_SIZE as usize {
                break;
            }
            // If no records in this batch could be resolved, break to avoid infinite loop
            if batch_updated == 0 {
                log::warn!(
                    "backfill_costs: {} records have no matching price, skipping",
                    records.len()
                );
                break;
            }
        }

        if total_count > 0 {
            log::info!("Backfilled cost for {} records", total_count);
        }
        Ok(total_count)
    }

    /// Backfill costs for a specific set of newly inserted records
    fn backfill_costs_for_records(&self, records: &[UsageRecord]) -> AppResult<()> {
        let needs_calc: Vec<&UsageRecord> = records
            .iter()
            .filter(|r| r.cost_usd.is_none())
            .collect();

        if needs_calc.is_empty() {
            return Ok(());
        }

        let price_map = self.build_price_map();
        for record in needs_calc {
            if let Some(price) = Self::find_price(&price_map, &record.model, &record.source) {
                let cost = calculate_cost(&record.tokens, price);
                self.db.update_record_cost(&record.id, cost)?;
            }
        }
        Ok(())
    }

    /// Find the best matching price for a model + source combination
    fn find_price<'a>(
        price_map: &'a HashMap<String, ModelPricing>,
        model: &str,
        _source: &DataSource,
    ) -> Option<&'a ModelPricing> {
        // Try exact match first (deterministic O(1) lookup)
        if let Some(price) = price_map.get(model) {
            return Some(price);
        }

        // Try case-insensitive match
        let model_lower = model.to_lowercase();
        let mut case_match: Option<&ModelPricing> = None;
        for (key, price) in price_map {
            if key.to_lowercase() == model_lower {
                case_match = Some(price);
                break;
            }
        }
        if let Some(price) = case_match {
            return Some(price);
        }

        // Try partial match — collect all matches and pick the most specific (longest key)
        let mut partial_matches: Vec<(&String, &ModelPricing)> = price_map
            .iter()
            .filter(|(key, _)| {
                let key_lower = key.to_lowercase();
                model_lower.contains(&key_lower) || key_lower.contains(&model_lower)
            })
            .collect();

        if partial_matches.is_empty() {
            return None;
        }

        // Sort by key length descending (longest = most specific match wins)
        partial_matches.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        Some(partial_matches[0].1)
    }
}
