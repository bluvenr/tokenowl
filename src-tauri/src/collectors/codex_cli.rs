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

#[derive(Deserialize)]
struct CodexJsonlLine {
    #[serde(rename = "type")]
    event_type: Option<String>,
    model: Option<String>,
    #[serde(rename = "last_token_usage")]
    last_token_usage: Option<CodexTokenUsage>,
    timestamp: Option<String>,
}

#[derive(Deserialize)]
struct CodexTokenUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    total_tokens: u64,
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
            return Ok(records);
        }
        for entry in walkdir::WalkDir::new(&self.base_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "jsonl") {
                match self.parse_file(path) {
                    Ok(r) => records.extend(r),
                    Err(e) => log::warn!("Codex parse error {:?}: {}", path, e),
                }
            }
        }
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
            if let Ok(parsed) = serde_json::from_str::<CodexJsonlLine>(trimmed) {
                if parsed.event_type.as_deref() == Some("turn.completed") {
                    if let (Some(model), Some(usage)) = (parsed.model, parsed.last_token_usage) {
                        let ts = parsed
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
                            model,
                            tokens: TokenUsage {
                                input_tokens: usage.input_tokens,
                                output_tokens: usage.output_tokens,
                                total_tokens: usage.total_tokens,
                                ..Default::default()
                            },
                            cost_usd: None,
                            project_path: None,
                        });
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
