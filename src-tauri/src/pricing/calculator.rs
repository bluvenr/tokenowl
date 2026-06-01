use crate::models::usage::TokenUsage;
use crate::models::settings::ModelPricing;

/// Calculate cost for a single usage record based on model pricing
pub fn calculate_cost(tokens: &TokenUsage, price: &ModelPricing) -> f64 {
    let input_cost = tokens.input_tokens as f64 * price.input_per_million / 1_000_000.0;
    let output_cost = tokens.output_tokens as f64 * price.output_per_million / 1_000_000.0;

    let cache_write_cost = tokens.cache_creation_tokens as f64
        * price.cache_write_per_million.unwrap_or(price.input_per_million)
        / 1_000_000.0;
    let cache_read_cost = tokens.cache_read_tokens as f64
        * price.cache_read_per_million.unwrap_or(price.input_per_million * 0.1)
        / 1_000_000.0;

    // Reasoning tokens: use dedicated price if set, otherwise fall back to output price
    let reasoning_cost = tokens.reasoning_tokens as f64
        * price.reasoning_per_million.unwrap_or(price.output_per_million)
        / 1_000_000.0;

    input_cost + output_cost + cache_write_cost + cache_read_cost + reasoning_cost
}
