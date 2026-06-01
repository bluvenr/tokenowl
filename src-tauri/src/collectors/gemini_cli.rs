use crate::collectors::traits::Collector;
use crate::error::AppResult;
use crate::models::usage::*;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

pub struct GeminiCliCollector {
    base_dir: PathBuf,
}

impl GeminiCliCollector {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            base_dir: home.join(".gemini").join("tmp"),
        }
    }
}

#[derive(Deserialize)]
struct GeminiSession {
    model: Option<String>,
    #[serde(default)]
    tokens: GeminiTokenUsage,
    timestamp: Option<String>,
}

#[derive(Deserialize, Default)]
struct GeminiTokenUsage {
    #[serde(default)]
    input: u64,
    #[serde(default)]
    output: u64,
    #[serde(default)]
    cached: u64,
}

impl Collector for GeminiCliCollector {
    fn source(&self) -> DataSource {
        DataSource::GeminiCli
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

    fn session_id_for_file(&self, path: &std::path::Path) -> String {
        path.parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default()
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
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(session) = serde_json::from_str::<GeminiSession>(&content) {
                        let total = session.tokens.input + session.tokens.output + session.tokens.cached;
                        let session_id = path
                            .parent()
                            .and_then(|p| p.file_name())
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let ts = session
                            .timestamp
                            .as_deref()
                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(Utc::now);
                        let model = session.model.unwrap_or_else(|| "gemini".to_string());
                        let file_key = path.to_string_lossy().to_string();
                        let id = super::make_record_id(
                            &DataSource::GeminiCli,
                            &session_id,
                            &ts.to_rfc3339(),
                            &model,
                            &file_key,
                        );
                        records.push(UsageRecord {
                            id,
                            source: DataSource::GeminiCli,
                            session_id,
                            timestamp: ts,
                            model,
                            tokens: TokenUsage {
                                input_tokens: session.tokens.input,
                                output_tokens: session.tokens.output,
                                cache_read_tokens: session.tokens.cached,
                                total_tokens: total,
                                ..Default::default()
                            },
                            cost_usd: None,
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
        // JSON files are re-read entirely on change
        let content = fs::read_to_string(file_path)?;
        let file_size = content.len() as u64;
        let records = if let Ok(session) = serde_json::from_str::<GeminiSession>(&content) {
            let total = session.tokens.input + session.tokens.output + session.tokens.cached;
            let session_id = file_path
                .parent()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let ts = session
                .timestamp
                .as_deref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);
            let model = session.model.unwrap_or_else(|| "gemini".to_string());
            let file_key = file_path.to_string_lossy().to_string();
            let id = super::make_record_id(
                &DataSource::GeminiCli,
                &session_id,
                &ts.to_rfc3339(),
                &model,
                &file_key,
            );
            vec![UsageRecord {
                id,
                source: DataSource::GeminiCli,
                session_id,
                timestamp: ts,
                model,
                tokens: TokenUsage {
                    input_tokens: session.tokens.input,
                    output_tokens: session.tokens.output,
                    cache_read_tokens: session.tokens.cached,
                    total_tokens: total,
                    ..Default::default()
                },
                cost_usd: None,
                project_path: None,
            }]
        } else {
            vec![]
        };
        Ok((records, file_size))
    }
}
