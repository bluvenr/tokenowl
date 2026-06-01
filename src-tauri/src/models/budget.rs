use serde::{Deserialize, Serialize};

/// Budget configuration (single-row config)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetConfig {
    pub daily_limit_usd: Option<f64>,
    pub weekly_limit_usd: Option<f64>,
    pub monthly_limit_usd: Option<f64>,
    pub alert_threshold_pct: u8,
    pub alert_icon_color: bool,
    pub alert_system_notify: bool,
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
        }
    }
}

/// Budget alert state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetAlert {
    pub triggered: bool,
    pub message: String,
    pub current_cost_usd: f64,
    pub limit_usd: f64,
    pub percentage: f64,
    pub period: String,
}
