use serde::{Deserialize, Serialize};

/// Application settings (single row)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub language: String,
    pub download_source: String,
    pub auto_start: bool,
    pub theme: String,
    pub tray_display: String,
    pub telemetry_enabled: bool,
    pub crash_log_enabled: bool,
    pub anomaly_threshold: f64,
    pub forecast_method: String,
    pub data_retention_days: u32,
    pub daily_digest_enabled: bool,
    pub daily_digest_time: String,
    pub weekly_digest_enabled: bool,
    pub update_check_interval_hours: u32,
    pub price_sync_interval_hours: u32,
    pub default_period: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "auto".to_string(),
            download_source: "auto".to_string(),
            auto_start: false,
            theme: "system".to_string(),
            tray_display: "cost".to_string(),
            telemetry_enabled: false,
            crash_log_enabled: true,
            anomaly_threshold: 2.5,
            forecast_method: "linear".to_string(),
            data_retention_days: 90,
            daily_digest_enabled: false,
            daily_digest_time: "20:00".to_string(),
            weekly_digest_enabled: false,
            update_check_interval_hours: 4,
            price_sync_interval_hours: 12,
            default_period: "week".to_string(),
        }
    }
}

/// Model pricing definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub model_id: String,
    pub display_name: String,
    pub input_per_million: f64,
    pub output_per_million: f64,
    pub cache_write_per_million: Option<f64>,
    pub cache_read_per_million: Option<f64>,
    pub source: PriceSource,
}

/// Price source priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PriceSource {
    Builtin,
    Remote,
    Custom,
}

/// Custom price override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPrice {
    pub model_id: String,
    pub input_per_million: f64,
    pub output_per_million: f64,
    pub cache_write_per_million: Option<f64>,
    pub cache_read_per_million: Option<f64>,
}
