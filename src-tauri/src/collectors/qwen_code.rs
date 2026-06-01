use crate::collectors::traits::Collector;
use crate::error::AppResult;
use crate::models::usage::*;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

pub struct QwenCodeCollector {
    base_dir: PathBuf,
}

impl QwenCodeCollector {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            base_dir: home.join(".qwen").join("history"),
        }
    }
}

#[derive(Deserialize)]
struct QwenSession {
    model: Option<String>,
    #[serde(default)]
    tokens: QwenTokenUsage,
    timestamp: Option<String>,
    #[serde(default)]
    cost: Option<f64>,
}

#[derive(Deserialize, Default)]
struct QwenTokenUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    total_tokens: u64,
}

impl Collector for QwenCodeCollector {
    fn source(&self) -> DataSource {
        DataSource::QwenCode
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
            if entry.path().extension().map_or(false, |ext| ext == "json") {
                files.push(entry.path().to_path_buf());
            }
        }
        Ok(files)
    }

    fn file_extensions(&self) -> &[&str] {
        &["json"]
    }

    fn is_whole_file(&self) -> bool { true }

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
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(session) = serde_json::from_str::<QwenSession>(&content) {
                        let total = if session.tokens.total_tokens > 0 {
                            session.tokens.total_tokens
                        } else {
                            session.tokens.input_tokens + session.tokens.output_tokens
                        };
                        let session_id = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let ts = session
                            .timestamp
                            .as_deref()
                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(Utc::now);
                        let model = session.model.unwrap_or_else(|| "qwen".to_string());
                        let file_key = path.to_string_lossy().to_string();
                        let id = super::make_record_id(
                            &DataSource::QwenCode,
                            &session_id,
                            &ts.to_rfc3339(),
                            &model,
                            &file_key,
                        );
                        records.push(UsageRecord {
                            id,
                            source: DataSource::QwenCode,
                            session_id,
                            timestamp: ts,
                            model,
                            tokens: TokenUsage {
                                input_tokens: session.tokens.input_tokens,
                                output_tokens: session.tokens.output_tokens,
                                total_tokens: total,
                                ..Default::default()
                            },
                            cost_usd: session.cost,
                            project_path: None,
                        });
                    }
                }
            }
        }
        Ok(records)
    }

    fn incremental_parse(
        &self,
        file_path: &PathBuf,
        _from_offset: u64,
    ) -> AppResult<(Vec<UsageRecord>, u64)> {
        // Qwen JSON files are re-read on change
        let content = fs::read_to_string(file_path)?;
        let file_size = content.len() as u64;
        let records = if let Ok(session) = serde_json::from_str::<QwenSession>(&content) {
            let total = if session.tokens.total_tokens > 0 {
                session.tokens.total_tokens
            } else {
                session.tokens.input_tokens + session.tokens.output_tokens
            };
            let session_id = file_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let ts = session
                .timestamp
                .as_deref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);
            let model = session.model.unwrap_or_else(|| "qwen".to_string());
            let file_key = file_path.to_string_lossy().to_string();
            let id = super::make_record_id(
                &DataSource::QwenCode,
                &session_id,
                &ts.to_rfc3339(),
                &model,
                &file_key,
            );
            vec![UsageRecord {
                id,
                source: DataSource::QwenCode,
                session_id,
                timestamp: ts,
                model,
                tokens: TokenUsage {
                    input_tokens: session.tokens.input_tokens,
                    output_tokens: session.tokens.output_tokens,
                    total_tokens: total,
                    ..Default::default()
                },
                cost_usd: session.cost,
                project_path: None,
            }]
        } else {
            vec![]
        };
        Ok((records, file_size))
    }
}
