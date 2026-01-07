// core/src/aggregator.rs

use crate::config::{AggregationLogic, AggregationRule};
use anyhow::Result;
use glob::glob;
use serde_yaml::Value;
use std::fs;
use std::path::Path;
use tracing::info;

pub struct Aggregator;

impl Aggregator {
    pub fn calculate(
        root_path: &Path,
        rules: &[AggregationRule],
    ) -> Result<std::collections::HashMap<String, f64>> {
        let mut results = std::collections::HashMap::new();

        for rule in rules {
            let mut total = 0.0;
            let mut count = 0.0;

            let search_pattern = root_path.join(&rule.path);
            let search_pattern_str = search_pattern.to_string_lossy();

            info!(
                "Deep Scan: Searching '{}' for rule '{}'",
                search_pattern_str, rule.name
            );

            for path in glob(&search_pattern_str).unwrap_or_else(|_| glob("").unwrap()).flatten() {
                if path.is_file() {
                    if let Some(val) = Self::extract_value(&path, &rule.target_field) {
                        total += val;
                        count += 1.0;
                    }
                }
            }

            let final_value = match rule.logic {
                AggregationLogic::Sum => total,
                AggregationLogic::Count => count,
                AggregationLogic::Average => {
                    if count > 0.0 {
                        total / count
                    } else {
                        0.0
                    }
                }
            };

            results.insert(rule.name.clone(), final_value);
        }

        Ok(results)
    }

    fn extract_value(path: &Path, field: &str) -> Option<f64> {
        let content = fs::read_to_string(path).ok()?;
        let yaml: Value = serde_yaml::from_str(&content).ok()?;

        yaml.get(field).and_then(|v| {
            if let Some(f) = v.as_f64() {
                Some(f)
            } else {
                v.as_i64().map(|i| i as f64)
            }
        })
    }
}

// ============== UNIT TESTS ==============
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_extract_value_from_yaml() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.yaml");

        // Slučaj 1: Vrijednost je float
        fs::write(&file_path, "amount: 123.45").unwrap();
        assert_eq!(
            Aggregator::extract_value(&file_path, "amount"),
            Some(123.45)
        );

        // Slučaj 2: Vrijednost je integer
        fs::write(&file_path, "amount: 500").unwrap();
        assert_eq!(Aggregator::extract_value(&file_path, "amount"), Some(500.0));

        // Slučaj 3: Polje ne postoji
        fs::write(&file_path, "total: 999").unwrap();
        assert_eq!(Aggregator::extract_value(&file_path, "amount"), None);

        // Slučaj 4: Vrijednost nije broj
        fs::write(&file_path, "amount: 'hello'").unwrap();
        assert_eq!(Aggregator::extract_value(&file_path, "amount"), None);
    }
}
