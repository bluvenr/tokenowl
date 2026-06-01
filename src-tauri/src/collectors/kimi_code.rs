use crate::collectors::traits::Collector;
use crate::error::AppResult;
use crate::models::usage::*;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;

pub struct KimiCodeCollector {
    base_dir: PathBuf,
}

impl KimiCodeCollector {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            base_dir: home.join(".kimi"),
        }
    }
}

#[derive(Deserialize)]
struct KimiContextLine {
    model: Option<String>,
    usage: Option<KimiUsage>,
    timestamp: Option<String>,
}

#[derive(Deserialize)]
struct KimiUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    total_tokens: u64,
}

impl Collector for KimiCodeCollector {
    fn source(&self) -> DataSource {
        DataSource::KimiCode
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
            if entry.path().file_name().map_or(false, |n| n == "context.jsonl") {
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
            if path.file_name().map_or(false, |n| n == "context.jsonl") {
                match self.parse_file(path) {
                    Ok(r) => records.extend(r),
                    Err(e) => log::warn!("Kimi parse error {:?}: {}", path, e),
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
            .parent()
            .and_then(|p| p.file_name())
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
            if let Ok(parsed) = serde_json::from_str::<KimiContextLine>(trimmed) {
                if let Some(usage) = parsed.usage {
                    let ts = parsed
                        .timestamp
                        .as_deref()
                        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(Utc::now);
                    let model = parsed.model.unwrap_or_else(|| "kimi".to_string());
                    let id = super::make_record_id(
                        &DataSource::KimiCode,
                        &session_id,
                        &ts.to_rfc3339(),
                        &model,
                        &offset_before.to_string(),
                    );
                    records.push(UsageRecord {
                        id,
                        source: DataSource::KimiCode,
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
        let final_offset = reader.stream_position().unwrap_or(from_offset);
        Ok((records, final_offset))
    }
}

impl KimiCodeCollector {
    fn parse_file(&self, path: &std::path::Path) -> AppResult<Vec<UsageRecord>> {
        let (records, _) = self.incremental_parse(&path.to_path_buf(), 0)?;
        Ok(records)
    }
}
