use crate::collectors::traits::Collector;
use crate::error::AppResult;
use crate::models::usage::*;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;

pub struct ClaudeCodeCollector {
    base_dir: PathBuf,
}

impl ClaudeCodeCollector {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            base_dir: home.join(".claude").join("projects"),
        }
    }
}

/// The `message` wrapper inside a Claude Code JSONL line.
/// For `type: "assistant"` events, `model` and `usage` live here.
#[derive(Deserialize)]
struct ClaudeCodeMessage {
    model: Option<String>,
    usage: Option<ClaudeCodeUsage>,
}

/// Raw JSONL line from Claude Code.
/// Model and usage data are nested inside the `message` field, NOT at top level.
#[derive(Deserialize)]
struct ClaudeCodeJsonlLine {
    #[serde(rename = "type")]
    event_type: Option<String>,
    message: Option<ClaudeCodeMessage>,
    timestamp: Option<String>,
}

#[derive(Deserialize)]
struct ClaudeCodeUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
}

impl Collector for ClaudeCodeCollector {
    fn source(&self) -> DataSource {
        DataSource::ClaudeCode
    }

    fn watch_paths(&self) -> Vec<PathBuf> {
        vec![self.base_dir.clone()]
    }

    fn is_available(&self) -> bool {
        self.base_dir.exists()
    }

    fn scanned_files(&self) -> AppResult<Vec<PathBuf>> {
        let mut files = Vec::new();
        if !self.base_dir.exists() {
            return Ok(files);
        }
        for entry in walkdir::WalkDir::new(&self.base_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().map_or(false, |ext| ext == "jsonl") {
                files.push(entry.path().to_path_buf());
            }
        }
        Ok(files)
    }

    fn file_extensions(&self) -> &[&str] {
        &["jsonl"]
    }

    fn full_scan(&self) -> AppResult<Vec<UsageRecord>> {
        let mut records = Vec::new();
        if !self.base_dir.exists() {
            log::info!("[Claude Code] Base dir not found: {:?}", self.base_dir);
            return Ok(records);
        }
        let mut file_count = 0u32;
        for entry in walkdir::WalkDir::new(&self.base_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "jsonl") {
                file_count += 1;
                match self.parse_file(path) {
                    Ok(file_records) => {
                        log::info!(
                            "[Claude Code] Parsed {:?}: {} records",
                            path.file_name().unwrap_or_default(),
                            file_records.len()
                        );
                        records.extend(file_records);
                    }
                    Err(e) => {
                        log::warn!("Failed to parse {:?}: {}", path, e);
                    }
                }
            }
        }
        log::info!(
            "[Claude Code] Full scan: {} files found, {} total records",
            file_count,
            records.len()
        );
        Ok(records)
    }

    fn incremental_parse(
        &self,
        file_path: &PathBuf,
        from_offset: u64,
    ) -> AppResult<(Vec<UsageRecord>, u64)> {
        let mut file = fs::File::open(file_path)?;
        file.seek(SeekFrom::Start(from_offset))?;

        let mut reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_buf = String::new();
        let session_id = extract_session_id(file_path);

        loop {
            line_buf.clear();
            let offset_before = reader.stream_position().unwrap_or(from_offset);
            let bytes_read = reader.read_line(&mut line_buf)?;
            if bytes_read == 0 {
                break;
            }

            let trimmed = line_buf.trim_end_matches(|c| c == '\n' || c == '\r');
            if let Ok(parsed) = serde_json::from_str::<ClaudeCodeJsonlLine>(trimmed) {
                // Only process "assistant" events — that's where model + usage live
                if parsed.event_type.as_deref() != Some("assistant") {
                    continue;
                }
                if let Some(msg) = parsed.message {
                    if let (Some(model), Some(usage)) = (msg.model, msg.usage) {
                        let ts = parse_timestamp(parsed.timestamp.as_deref());
                        let id = super::make_record_id(
                            &DataSource::ClaudeCode,
                            &session_id,
                            &ts.to_rfc3339(),
                            &model,
                            &offset_before.to_string(),
                        );
                        let record = UsageRecord {
                            id,
                            source: DataSource::ClaudeCode,
                            session_id: session_id.clone(),
                            timestamp: ts,
                            model,
                            tokens: TokenUsage {
                                input_tokens: usage.input_tokens,
                                output_tokens: usage.output_tokens,
                                cache_creation_tokens: usage.cache_creation_input_tokens,
                                cache_read_tokens: usage.cache_read_input_tokens,
                                total_tokens: usage.input_tokens
                                    + usage.output_tokens
                                    + usage.cache_creation_input_tokens
                                    + usage.cache_read_input_tokens,
                                reasoning_tokens: 0,
                            },
                            cost_usd: None,
                            project_path: extract_project_path(file_path),
                        };
                        records.push(record);
                    }
                }
            }
        }
        let final_offset = reader.stream_position().unwrap_or(from_offset);
        Ok((records, final_offset))
    }
}

impl ClaudeCodeCollector {
    fn parse_file(&self, path: &std::path::Path) -> AppResult<Vec<UsageRecord>> {
        let (records, _) = self.incremental_parse(&path.to_path_buf(), 0)?;
        Ok(records)
    }
}

/// Extract session ID from file path
fn extract_session_id(path: &std::path::Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Extract project path from the directory structure
/// ~/.claude/projects/<project-hash>/<session>.jsonl
fn extract_project_path(path: &std::path::Path) -> Option<String> {
    path.parent()
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().to_string())
}

/// Parse timestamp string to DateTime<Utc>
fn parse_timestamp(ts: Option<&str>) -> DateTime<Utc> {
    ts.and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now)
}
