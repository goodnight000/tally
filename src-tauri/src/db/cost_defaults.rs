use std::collections::HashMap;
use rusqlite::{params, Connection};
use serde::Deserialize;

/// Embedded model pricing data from LiteLLM (fetched at build time)
const MODEL_PRICES_JSON: &str = include_str!("../model_prices.json");

#[derive(Debug, Deserialize)]
struct PriceEntry {
    input_per_million: f64,
    output_per_million: f64,
    cache_read_per_million: Option<f64>,
    cache_creation_per_million: Option<f64>,
}

/// Load the embedded pricing database
fn load_price_map() -> HashMap<String, PriceEntry> {
    serde_json::from_str(MODEL_PRICES_JSON).unwrap_or_default()
}

/// Look up pricing for a model name, with fuzzy matching.
/// Tries exact match first, then progressively shorter prefixes.
pub fn lookup_model_price(model: &str) -> Option<(f64, f64, Option<f64>, Option<f64>)> {
    let prices = load_price_map();

    // Exact match
    if let Some(entry) = prices.get(model) {
        return Some((
            entry.input_per_million,
            entry.output_per_million,
            entry.cache_read_per_million,
            entry.cache_creation_per_million,
        ));
    }

    // Try without provider prefix (e.g., "anthropic/" or "openai/")
    let without_prefix = if let Some(idx) = model.find('/') {
        &model[idx + 1..]
    } else {
        model
    };
    if without_prefix != model {
        if let Some(entry) = prices.get(without_prefix) {
            return Some((
                entry.input_per_million,
                entry.output_per_million,
                entry.cache_read_per_million,
                entry.cache_creation_per_million,
            ));
        }
    }

    // Try adding "openai/" prefix (Codex models stored as "gpt-5.4" but LiteLLM uses "openai/gpt-5.4")
    if !model.contains('/') {
        let with_prefix = format!("openai/{}", model);
        if let Some(entry) = prices.get(&with_prefix) {
            return Some((
                entry.input_per_million,
                entry.output_per_million,
                entry.cache_read_per_million,
                entry.cache_creation_per_million,
            ));
        }
    }

    // Try matching by base model name (strip date suffix)
    let base = strip_date_suffix(model);
    if base != model {
        if let Some(entry) = prices.get(base) {
            return Some((
                entry.input_per_million,
                entry.output_per_million,
                entry.cache_read_per_million,
                entry.cache_creation_per_million,
            ));
        }
    }

    // Try partial match: find the longest key that our model starts with
    let mut best_match: Option<(&str, &PriceEntry)> = None;
    for (key, entry) in &prices {
        if model.starts_with(key.as_str()) || key.starts_with(model) {
            match &best_match {
                Some((prev_key, _)) if key.len() > prev_key.len() => {
                    best_match = Some((key, entry));
                }
                None => {
                    best_match = Some((key, entry));
                }
                _ => {}
            }
        }
    }
    if let Some((_, entry)) = best_match {
        return Some((
            entry.input_per_million,
            entry.output_per_million,
            entry.cache_read_per_million,
            entry.cache_creation_per_million,
        ));
    }

    None
}

/// Strip date suffix like "-20250514" from model names
fn strip_date_suffix(model: &str) -> &str {
    // Look for pattern like -YYYYMMDD at the end
    if model.len() >= 9 {
        let suffix = &model[model.len() - 9..];
        if suffix.starts_with('-') && suffix[1..].chars().all(|c| c.is_ascii_digit()) {
            return &model[..model.len() - 9];
        }
    }
    model
}

/// Auto-populate cost_rates table for any models found in sessions
/// that don't already have user-configured rates.
pub fn populate_default_costs(conn: &Connection) {
    // Get all distinct models from sessions that don't have cost rates yet
    let sql = "SELECT DISTINCT s.model FROM sessions s
               LEFT JOIN cost_rates c ON s.model = c.model
               WHERE s.model IS NOT NULL AND c.model IS NULL";

    let models: Vec<String> = conn
        .prepare(sql)
        .and_then(|mut stmt| {
            let rows = stmt.query_map([], |row| row.get(0))?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();

    let mut inserted = 0;
    for model in &models {
        if let Some((input, output, cache_read, cache_creation)) = lookup_model_price(model) {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO cost_rates
                 (model, input_per_million, output_per_million, cache_read_per_million, cache_creation_per_million)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![model, input, output, cache_read, cache_creation],
            );
            inserted += 1;
        }
    }

    if inserted > 0 {
        log::info!("Auto-populated cost rates for {} models", inserted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_price_map() {
        let map = load_price_map();
        assert!(!map.is_empty(), "Price map should not be empty");
    }

    #[test]
    fn test_lookup_exact_match() {
        // This model should exist in the embedded data
        let result = lookup_model_price("claude-3-5-sonnet-20241022");
        assert!(result.is_some());
        let (input, output, _, _) = result.unwrap();
        assert!(input > 0.0);
        assert!(output > 0.0);
    }

    #[test]
    fn test_lookup_fuzzy_match() {
        // Try a model with date suffix that might not be exact
        let result = lookup_model_price("claude-3-5-sonnet");
        assert!(result.is_some());
    }

    #[test]
    fn test_strip_date_suffix() {
        assert_eq!(strip_date_suffix("claude-sonnet-4-5-20250514"), "claude-sonnet-4-5");
        assert_eq!(strip_date_suffix("gpt-4o"), "gpt-4o");
        assert_eq!(strip_date_suffix("o3-mini"), "o3-mini");
    }

    #[test]
    fn test_populate_default_costs() {
        let conn = crate::db::init_test_db().unwrap();

        // Insert a session with a known model
        conn.execute(
            "INSERT INTO sessions (id, tool, start_time, model, total_tokens)
             VALUES ('test:1', 'claude', '2025-01-01T00:00:00Z', 'claude-3-5-sonnet-20241022', 1000)",
            [],
        ).unwrap();

        // Populate costs
        populate_default_costs(&conn);

        // Verify a rate was created
        let rate: Option<f64> = conn
            .query_row(
                "SELECT input_per_million FROM cost_rates WHERE model = 'claude-3-5-sonnet-20241022'",
                [],
                |row| row.get(0),
            )
            .ok();
        assert!(rate.is_some());
        assert!(rate.unwrap() > 0.0);
    }
}
