use crate::collectors::traits::Collector;
use crate::error::AppResult;
use crate::models::usage::*;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;

pub struct CodexCliCollector {
    base_dir: PathBuf,
}

impl CodexCliCollector {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            base_dir: home.join(".codex").join("sessions"),
        }
    }
}

// ── Deserialization structs for Codex CLI rollout JSONL ──────────────

/// Token usage from `payload.info.total_token_usage`
#[derive(Deserialize)]
#[allow(dead_code)]
struct CodexTokenCountInfo {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    cached_input_tokens: u64,
    #[serde(default)]
    reasoning_output_tokens: u64,
    #[serde(default)]
    total_tokens: u64,
}

/// Wrapper for `payload.info` which nests `total_token_usage` / `last_token_usage`
#[derive(Deserialize)]
#[allow(dead_code)]
struct CodexTokenCountInfoWrapper {
    #[serde(default)]
    total_token_usage: Option<CodexTokenCountInfo>,
    #[serde(default)]
    last_token_usage: Option<CodexTokenCountInfo>,
}

/// Payload for `type: "event_msg"` where `payload.type: "token_count"`
#[derive(Deserialize)]
struct CodexTokenCountPayload {
    #[serde(rename = "type")]
    payload_type: Option<String>,
    info: Option<CodexTokenCountInfoWrapper>,
}

/// Payload for `type: "session_meta"` — contains cwd/project info
#[derive(Deserialize)]
#[allow(dead_code)]
struct CodexSessionMetaPayload {
    cwd: Option<String>,
    model_provider: Option<String>,
}

/// Payload for `type: "turn_context"` — contains the actual model name
#[derive(Deserialize)]
struct CodexTurnContextPayload {
    model: Option<String>,
}

/// Generic raw JSONL line — uses serde_json::Value for payload to handle
/// the different payload shapes across event types.
#[derive(Deserialize)]
struct CodexRawLine {
    timestamp: Option<String>,
    #[serde(rename = "type")]
    event_type: Option<String>,
    payload: Option<serde_json::Value>,
}

impl Collector for CodexCliCollector {
    fn source(&self) -> DataSource {
        DataSource::CodexCli
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
            log::info!("[Codex CLI] Base dir not found: {:?}", self.base_dir);
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
                    Ok(r) => {
                        log::info!(
                            "[Codex CLI] Parsed {:?}: {} records",
                            path.file_name().unwrap_or_default(),
                            r.len()
                        );
                        records.extend(r);
                    }
                    Err(e) => log::warn!("Codex parse error {:?}: {}", path, e),
                }
            }
        }
        log::info!(
            "[Codex CLI] Full scan: {} files found, {} total records",
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
        // Pre-scan from file start to extract model and project info.
        // These live in `session_meta` / `turn_context` events at the top
        // of the file, which we'd miss if we only read from `from_offset`.
        let (model, project_path) = prescan_file_metadata(file_path)?;
        let model = model.unwrap_or_else(|| "unknown".to_string());

        let mut file = fs::File::open(file_path)?;
        file.seek(SeekFrom::Start(from_offset))?;
        let mut reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_buf = String::new();
        let session_id = file_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        loop {
            line_buf.clear();
            let offset_before = reader.stream_position().unwrap_or(from_offset);
            let bytes_read = reader.read_line(&mut line_buf)?;
            if bytes_read == 0 {
                break;
            }

            let trimmed = line_buf.trim_end_matches(|c| c == '\n' || c == '\r');
            if let Ok(raw) = serde_json::from_str::<CodexRawLine>(trimmed) {
                // We only care about `event_msg` with `payload.type == "token_count"`
                if raw.event_type.as_deref() != Some("event_msg") {
                    continue;
                }
                if let Some(payload_val) = raw.payload {
                    if let Ok(tc) =
                        serde_json::from_value::<CodexTokenCountPayload>(payload_val)
                    {
                        if tc.payload_type.as_deref() == Some("token_count") {
                            if let Some(info) = tc.info
                                .and_then(|w| w.total_token_usage)
                            {
                                let ts = raw
                                    .timestamp
                                    .as_deref()
                                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                                    .map(|dt| dt.with_timezone(&Utc))
                                    .unwrap_or_else(Utc::now);

                                let id = super::make_record_id(
                                    &DataSource::CodexCli,
                                    &session_id,
                                    &ts.to_rfc3339(),
                                    &model,
                                    &offset_before.to_string(),
                                );

                                records.push(UsageRecord {
                                    id,
                                    source: DataSource::CodexCli,
                                    session_id: session_id.clone(),
                                    timestamp: ts,
                                    model: model.clone(),
                                    tokens: TokenUsage {
                                        input_tokens: info.input_tokens,
                                        output_tokens: info.output_tokens,
                                        cache_creation_tokens: 0,
                                        cache_read_tokens: info.cached_input_tokens,
                                        total_tokens: info.total_tokens,
                                        reasoning_tokens: info.reasoning_output_tokens,
                                    },
                                    cost_usd: None,
                                    project_path: project_path.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
        let final_offset = reader.stream_position().unwrap_or(from_offset);
        Ok((records, final_offset))
    }
}

impl CodexCliCollector {
    fn parse_file(&self, path: &std::path::Path) -> AppResult<Vec<UsageRecord>> {
        let (records, _) = self.incremental_parse(&path.to_path_buf(), 0)?;
        Ok(records)
    }
}

/// Pre-scan the entire file to extract model name and project path (cwd).
/// These are stored in `turn_context` / `session_meta` events at the top
/// of the rollout file, before any token_count events appear.
fn prescan_file_metadata(path: &PathBuf) -> AppResult<(Option<String>, Option<String>)> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut model: Option<String> = None;
    let mut project_path: Option<String> = None;

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(raw) = serde_json::from_str::<CodexRawLine>(trimmed) {
            match raw.event_type.as_deref() {
                Some("session_meta") => {
                    if let Some(payload_val) = raw.payload {
                        if let Ok(meta) =
                            serde_json::from_value::<CodexSessionMetaPayload>(payload_val)
                        {
                            if project_path.is_none() {
                                project_path = meta.cwd;
                            }
                        }
                    }
                }
                Some("turn_context") => {
                    if let Some(payload_val) = raw.payload {
                        if let Ok(ctx) =
                            serde_json::from_value::<CodexTurnContextPayload>(payload_val)
                        {
                            if model.is_none() {
                                model = ctx.model;
                            }
                        }
                    }
                }
                _ => {}
            }
            // Once we have both pieces of info, stop scanning
            if model.is_some() && project_path.is_some() {
                break;
            }
        }
    }
    log::info!(
        "[Codex CLI] Prescan {:?}: model={}, cwd={}",
        path.file_name().unwrap_or_default(),
        model.as_deref().unwrap_or("none"),
        project_path.as_deref().unwrap_or("none"),
    );
    Ok((model, project_path))
}
