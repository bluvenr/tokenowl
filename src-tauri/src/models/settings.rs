use serde::{Deserialize, Serialize};

/// Application settings (single-row config)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub language: String,
    pub download_source: String,
    pub auto_start: bool,
    pub theme: String,
    pub tray_display: String,
    pub telemetry_enabled: bool,
    pub crash_log_enabled: bool,
    pub price_sync_interval_hours: u8,
    pub update_check_interval_hours: u8,
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
            price_sync_interval_hours: 12,
            update_check_interval_hours: 4,
        }
    }
}

/// Model pricing definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPricing {
    pub model_id: String,
    pub display_name: String,
    pub source: String,
    pub input_per_million: f64,
    pub output_per_million: f64,
    pub cache_write_per_million: Option<f64>,
    pub cache_read_per_million: Option<f64>,
    #[serde(default)]
    pub reasoning_per_million: Option<f64>,
    #[serde(default)]
    pub price_source: String, // "remote" | "cached" | "custom"
    /// Whether a non-custom fallback exists (remote / cached)
    #[serde(default)]
    pub has_default: bool,
    /// Creation timestamp for custom prices (ISO 8601)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// Data source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceConfig {
    pub source: String,
    pub enabled: bool,
    pub custom_path: Option<String>,
    #[serde(default)]
    pub available: bool,
    #[serde(default)]
    pub status: String, // "available" | "unavailable" | "collecting" | "error"
}
