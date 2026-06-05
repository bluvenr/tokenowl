use std::sync::Arc;
use tauri::State;
use serde::{Deserialize, Serialize};
use crate::storage::database::Database;
use crate::storage::queries;

/// Savings analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavingsAnalysis {
    pub cache_hit_rate: f64,
    pub cache_savings_usd: f64,
    pub herfindahl_index: f64,
    pub model_concentration: String,
    pub monthly_forecast_usd: f64,
    pub forecast_confidence: f64,
    pub recommendations: Vec<String>,
}

#[tauri::command]
pub async fn get_savings_analysis(
    db: State<'_, Arc<Database>>,
    period: String,
) -> Result<SavingsAnalysis, String> {
    // Get usage summary for the period
    let summary = queries::get_usage_summary(&db, &period).map_err(|e| e.to_string())?;

    // Cache hit rate: cache_tokens / (input_tokens + output_tokens + cache_tokens)
    let total_input = summary.input_tokens as f64;
    let total_output = summary.output_tokens as f64;
    let total_cache = summary.cache_tokens as f64;
    let all_tokens = total_input + total_output + total_cache;
    let cache_hit_rate = if all_tokens > 0.0 {
        total_cache / all_tokens * 100.0
    } else {
        0.0
    };

    // Estimated cache savings: cache_read_tokens * avg_input_price * 0.9 (cache is ~10% of input price)
    let cache_savings_usd = if all_tokens > 0.0 {
        let avg_cost_per_token = if summary.total_tokens > 0 {
            summary.total_cost_usd / summary.total_tokens as f64
        } else {
            0.0
        };
        total_cache * avg_cost_per_token * 0.9
    } else {
        0.0
    };

    // Herfindahl index for model concentration
    let model_usage = queries::get_usage_by_model(&db, &period).map_err(|e| e.to_string())?;
    let herfindahl_index = if !model_usage.is_empty() {
        let shares: Vec<f64> = model_usage.iter().map(|m| m.percentage / 100.0).collect();
        shares.iter().map(|s| s * s).sum::<f64>()
    } else {
        0.0
    };

    let model_concentration = if herfindahl_index > 0.5 {
        "high".to_string()
    } else if herfindahl_index > 0.25 {
        "moderate".to_string()
    } else {
        "diverse".to_string()
    };

    // Monthly forecast (simple linear extrapolation)
    let monthly_forecast_usd = match period.as_str() {
        "today" => summary.total_cost_usd * 30.0,
        "week" => summary.total_cost_usd * (30.0 / 7.0),
        "month" => summary.total_cost_usd,
        _ => summary.total_cost_usd * (30.0 / 7.0),
    };
    let forecast_confidence = match period.as_str() {
        "today" => 0.3,
        "week" => 0.6,
        "month" => 0.85,
        _ => 0.5,
    };

    // Generate recommendations
    let mut recommendations = Vec::new();

    if cache_hit_rate < 20.0 && summary.request_count > 50 {
        recommendations.push(
            "缓存命中率较低，建议在 CC Switch 中检查缓存配置，可节省大量费用".to_string()
        );
    }

    if herfindahl_index > 0.7 {
        recommendations.push(
            "模型集中度过高，考虑对低优先级任务使用更经济的模型以降低成本".to_string()
        );
    }

    let total_cost = summary.total_cost_usd;
    let daily_avg = match period.as_str() {
        "week" => total_cost / 7.0,
        "month" => total_cost / 30.0,
        _ => total_cost,
    };
    if daily_avg > 1.0 {
        recommendations.push(format!(
            "日均花费 ${:.2}，建议设置每日预算上限以控制支出",
            daily_avg
        ));
    }

    if model_usage.len() > 5 {
        recommendations.push(
            "使用的模型较多，建议统一主力模型以获得更好的缓存效果和批量折扣".to_string()
        );
    }

    if recommendations.is_empty() {
        recommendations.push("当前使用模式良好，暂无优化建议".to_string());
    }

    Ok(SavingsAnalysis {
        cache_hit_rate,
        cache_savings_usd,
        herfindahl_index,
        model_concentration,
        monthly_forecast_usd,
        forecast_confidence,
        recommendations,
    })
}
