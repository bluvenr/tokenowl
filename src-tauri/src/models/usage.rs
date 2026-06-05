use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unified usage record - all data from CC Switch normalized into this format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub app_type: String,
    pub provider_name: Option<String>,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub reasoning_tokens: u64,
    pub total_tokens: u64,
    pub cost_usd: Option<f64>,
    pub status_code: Option<u16>,
    pub response_time_ms: Option<u64>,
    pub cc_switch_log_id: String,
    pub created_at: DateTime<Utc>,
}

/// Token usage breakdown
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub reasoning_tokens: u64,
    pub total_tokens: u64,
}

/// Usage summary for a time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub total_cost_usd: f64,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_tokens: u64,
    pub session_count: u64,
    pub request_count: u64,
}

/// Usage breakdown by model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub model: String,
    pub cost_usd: f64,
    pub total_tokens: u64,
    pub request_count: u64,
    pub percentage: f64,
}

/// Usage breakdown by provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUsage {
    pub provider_name: String,
    pub cost_usd: f64,
    pub total_tokens: u64,
    pub request_count: u64,
    pub avg_latency_ms: f64,
    pub failure_rate: f64,
    pub percentage: f64,
}

/// Trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub timestamp: String,
    pub cost_usd: f64,
    pub total_tokens: u64,
}

/// Recent session/request summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub model: String,
    pub provider_name: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub cost_usd: Option<f64>,
    pub status_code: Option<u16>,
    pub response_time_ms: Option<u64>,
}

/// Period-over-period comparison data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodComparison {
    pub cost_change_pct: Option<f64>,
    pub tokens_change_pct: Option<f64>,
    pub requests_change_pct: Option<f64>,
    pub sessions_change_pct: Option<f64>,
}

/// A single anomalous day
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnomaly {
    pub date: String,
    pub cost_usd: f64,
    pub avg_cost: f64,
    pub deviation: f64,
    pub top_provider: Option<String>,
    pub top_model: Option<String>,
}

/// Cost anomaly report for a period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnomalyReport {
    pub anomaly_days: Vec<CostAnomaly>,
    pub total_days: u64,
    pub avg_daily_cost: f64,
    pub stddev: f64,
    pub threshold: f64,
}

/// Token type breakdown for cost attribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBreakdown {
    pub token_type: String,
    pub cost_usd: f64,
    pub tokens: u64,
    pub percentage: f64,
}

/// Model-level attribution within a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAttribution {
    pub model: String,
    pub cost_usd: f64,
    pub total_tokens: u64,
    pub token_breakdown: Vec<TokenBreakdown>,
    pub percentage: f64,
}

/// Provider-level cost attribution with nested models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAttribution {
    pub provider_name: String,
    pub cost_usd: f64,
    pub models: Vec<ModelAttribution>,
    pub percentage: f64,
}

/// Budget burn rate analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetBurnRate {
    pub daily_rate: f64,
    pub daily_spend: Option<f64>,
    pub daily_limit: Option<f64>,
    pub daily_days_remaining: Option<f64>,
    pub weekly_spend: Option<f64>,
    pub weekly_limit: Option<f64>,
    pub weekly_days_remaining: Option<f64>,
    pub monthly_spend: Option<f64>,
    pub monthly_limit: Option<f64>,
    pub monthly_days_remaining: Option<f64>,
}

/// Cache trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheTrendPoint {
    pub timestamp: String,
    pub cache_hit_rate: f64,
    pub cache_tokens: u64,
    pub total_tokens: u64,
}
