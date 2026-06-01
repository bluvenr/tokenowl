use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unified usage record — the final normalized format from all data sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub id: String,
    pub source: DataSource,
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub model: String,
    pub tokens: TokenUsage,
    pub cost_usd: Option<f64>,
    pub project_path: Option<String>,
}

/// Token usage breakdown
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub total_tokens: u64,
    #[serde(default)]
    pub reasoning_tokens: u64,
}

/// Supported data sources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DataSource {
    ClaudeCode,
    CodexCli,
    GeminiCli,
    KimiCode,
    QwenCode,
}

impl DataSource {
    pub fn display_name(&self) -> &str {
        match self {
            DataSource::ClaudeCode => "Claude Code",
            DataSource::CodexCli => "Codex CLI",
            DataSource::GeminiCli => "Gemini CLI",
            DataSource::KimiCode => "Kimi Code",
            DataSource::QwenCode => "Qwen Code",
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            DataSource::ClaudeCode => "claude_code",
            DataSource::CodexCli => "codex_cli",
            DataSource::GeminiCli => "gemini_cli",
            DataSource::KimiCode => "kimi_code",
            DataSource::QwenCode => "qwen_code",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "claude_code" => Some(DataSource::ClaudeCode),
            "codex_cli" => Some(DataSource::CodexCli),
            "gemini_cli" => Some(DataSource::GeminiCli),
            "kimi_code" => Some(DataSource::KimiCode),
            "qwen_code" => Some(DataSource::QwenCode),
            _ => None,
        }
    }

    /// Default watch paths relative to home directory
    pub fn default_watch_paths(&self) -> Vec<&str> {
        match self {
            DataSource::ClaudeCode => vec![".claude/projects"],
            DataSource::CodexCli => vec![".codex/sessions"],
            DataSource::GeminiCli => vec![".gemini/tmp"],
            DataSource::KimiCode => vec![".kimi"],
            DataSource::QwenCode => vec![".qwen/history"],
        }
    }
}

/// Summary for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSummary {
    pub total_cost_usd: f64,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_tokens: u64,
    pub session_count: u64,
}

/// Per-source usage breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceUsage {
    pub source: String,
    pub display_name: String,
    pub cost_usd: f64,
    pub total_tokens: u64,
    pub percentage: f64,
}

/// Per-model usage breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    pub model: String,
    pub source: String,
    pub cost_usd: f64,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_tokens: u64,
}

/// Trend data point for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendPoint {
    pub date: String,
    pub cost_usd: f64,
    pub total_tokens: u64,
}

/// Session summary for list view
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub session_id: String,
    pub source: String,
    pub model: String,
    pub cost_usd: f64,
    pub total_tokens: u64,
    pub timestamp: String,
    pub project_path: Option<String>,
}

/// A model that has usage records but no price configured
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissingModelPrice {
    pub model: String,
    pub source: String,
}
