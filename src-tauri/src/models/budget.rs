use serde::{Deserialize, Serialize};

/// Budget configuration (single row)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    pub daily_limit_usd: Option<f64>,
    pub weekly_limit_usd: Option<f64>,
    pub monthly_limit_usd: Option<f64>,
    pub alert_threshold_pct: u8,
    pub alert_icon_color: bool,
    pub alert_system_notify: bool,
    pub alert_dashboard_banner: bool,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            daily_limit_usd: None,
            weekly_limit_usd: None,
            monthly_limit_usd: None,
            alert_threshold_pct: 80,
            alert_icon_color: true,
            alert_system_notify: true,
            alert_dashboard_banner: true,
        }
    }
}

/// Budget alert status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    pub triggered: bool,
    pub period: String,
    pub current_cost: f64,
    pub limit: f64,
    pub percentage: f64,
    pub message: String,
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbStats {
    pub total_records: u64,
    pub total_cost_usd: f64,
    pub date_range_start: Option<String>,
    pub date_range_end: Option<String>,
    pub db_size_bytes: u64,
}
