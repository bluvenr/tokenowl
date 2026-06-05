use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unified usage record — the final normalized format from CC Switch data
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
    /// Provider name from CC Switch (e.g., "PackyCode", "AIGoCode")
    pub provider_name: Option<String>,
    /// HTTP response time in milliseconds
    pub response_time_ms: Option<u64>,
    /// HTTP status code
    pub status_code: Option<u16>,
    /// CC Switch original log ID for deduplication
    pub cc_switch_log_id: Option<String>,
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

/// The single data source — CC Switch
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DataSource {
    CcSwitch,
}

impl DataSource {
    pub fn display_name(&self) -> &str {
        match self {
            DataSource::CcSwitch => "CC Switch",
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            DataSource::CcSwitch => "ccswitch",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ccswitch" => Some(DataSource::CcSwitch),
            _ => None,
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

// ─── Savings Engine Models ───────────────────────────────────────────

/// Cache efficiency for a single data source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheEfficiency {
    pub source: String,
    pub display_name: String,
    pub total_cache_read: u64,
    pub total_cache_creation: u64,
    pub total_input: u64,
    /// Cache hit rate: cache_read / (cache_read + input). None if no cache data.
    pub hit_rate: Option<f64>,
    /// Estimated savings from cache using actual model prices (cache_read * (input_price - cache_read_price))
    pub cache_cost_savings: f64,
}

/// Insight for a single model's usage pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsageInsight {
    pub model: String,
    pub source: String,
    pub cost_usd: f64,
    pub total_tokens: u64,
    pub session_count: u64,
    /// Average cost per session
    pub cost_per_session: f64,
    /// Cost per million tokens (based on actual spending)
    pub cost_per_million_tokens: f64,
    /// Percentage of total cost
    pub cost_share_pct: f64,
}

/// Model usage analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelAnalysis {
    pub insights: Vec<ModelUsageInsight>,
    /// Highest spending model
    pub top_cost_model: Option<ModelUsageInsight>,
    /// Top model's cost share percentage
    pub top_cost_share_pct: f64,
    /// Herfindahl concentration index (0-1, closer to 1 = more concentrated)
    pub concentration_index: f64,
}

/// Cost forecast based on current spending velocity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostForecast {
    /// Average daily cost this month
    pub daily_avg_cost: f64,
    /// Projected total cost by end of month
    pub projected_monthly_cost: f64,
    /// Days remaining in current month
    pub days_remaining: u32,
    /// Days elapsed in current month
    pub days_elapsed: u32,
    /// Monthly budget limit (if set)
    pub monthly_limit: Option<f64>,
    /// Whether projected cost exceeds budget
    pub projected_over_budget: bool,
    /// Estimated days until budget exhausted (None if no budget or on track)
    pub budget_exhaustion_days: Option<u32>,
    /// Week-over-week percentage change (None if insufficient data)
    pub week_over_week_change_pct: Option<f64>,
}

/// A detected cost anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostAnomaly {
    pub date: String,
    pub cost_usd: f64,
    pub daily_avg: f64,
    /// Deviation factor (e.g., 3.2 = 3.2x the daily average)
    pub deviation_factor: f64,
    /// Primary contributing source (optional)
    pub source: Option<String>,
}

/// Anomaly detection report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnomalyReport {
    /// Anomalies sorted by deviation factor (descending)
    pub anomalies: Vec<CostAnomaly>,
    pub daily_avg_cost: f64,
    pub daily_std_dev: f64,
    /// Threshold factor used (default 2.5)
    pub threshold_factor: f64,
}

/// Complete savings analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavingsAnalysis {
    pub cache_efficiency: Vec<CacheEfficiency>,
    pub model_analysis: ModelAnalysis,
    pub forecast: CostForecast,
    pub anomaly_report: AnomalyReport,
}

// ─── CC Switch Specific Models ───────────────────────────────────────

/// Provider efficiency analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInsight {
    pub provider_name: String,
    pub total_cost: f64,
    pub request_count: u64,
    pub avg_latency_ms: f64,
    pub failure_rate_pct: f64,
    pub cost_per_request: f64,
}

/// Sync state between TokenOwl and CC Switch
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncState {
    pub last_sync_time: Option<String>,
    pub last_sync_record_count: u64,
    pub cc_switch_db_path: String,
    pub cc_switch_detected: bool,
    pub sync_interval_secs: u64,
    pub total_records_synced: u64,
}

/// Result of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub new_records: u64,
    pub skipped_duplicates: u64,
    pub errors: u64,
    pub sync_duration_ms: u64,
}

/// CC Switch connection info (returned by detect)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CcSwitchInfo {
    pub db_path: String,
    pub db_size_bytes: u64,
    pub record_count: u64,
}

/// CC Switch status (connection + sync combined)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CcSwitchStatus {
    pub detected: bool,
    pub db_path: Option<String>,
    pub db_size_bytes: Option<u64>,
    pub record_count: Option<u64>,
    pub is_running: bool,
    pub sync_state: SyncState,
}
