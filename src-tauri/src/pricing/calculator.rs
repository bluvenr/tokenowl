use crate::models::usage::TokenUsage;
use crate::models::settings::ModelPricing;

/// Calculate cost for a given token usage and model pricing
pub fn calculate_cost(tokens: &TokenUsage, price: &ModelPricing) -> f64 {
    let input_cost = tokens.input_tokens as f64 * price.input_per_million / 1_000_000.0;
    let output_cost = tokens.output_tokens as f64 * price.output_per_million / 1_000_000.0;

    let cache_write_cost = if tokens.cache_creation_tokens > 0 {
        let rate = price.cache_write_per_million.unwrap_or(price.input_per_million);
        tokens.cache_creation_tokens as f64 * rate / 1_000_000.0
    } else {
        0.0
    };

    let cache_read_cost = if tokens.cache_read_tokens > 0 {
        let rate = price.cache_read_per_million.unwrap_or(price.input_per_million * 0.1);
        tokens.cache_read_tokens as f64 * rate / 1_000_000.0
    } else {
        0.0
    };

    input_cost + output_cost + cache_write_cost + cache_read_cost
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::settings::PriceSource;

    #[test]
    fn test_calculate_cost_basic() {
        let tokens = TokenUsage {
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
            reasoning_tokens: 0,
            total_tokens: 2_000_000,
        };

        let price = ModelPricing {
            model_id: "test".to_string(),
            display_name: "Test".to_string(),
            input_per_million: 3.0,
            output_per_million: 15.0,
            cache_write_per_million: None,
            cache_read_per_million: None,
            source: PriceSource::Builtin,
        };

        let cost = calculate_cost(&tokens, &price);
        assert!((cost - 18.0).abs() < 0.001);
    }
}
