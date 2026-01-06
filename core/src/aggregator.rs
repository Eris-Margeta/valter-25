use crate::config::{AggregationLogic, AggregationRule};
use anyhow::Result;
use glob::glob;
use serde_yaml::Value;
use std::fs;
use std::path::Path;
use tracing::{info, warn};

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

            if let Ok(paths) = glob(&search_pattern_str) {
                for entry in paths {
                    if let Ok(path) = entry {
                        if path.is_file() {
                            if let Some(val) = Self::extract_value(&path, &rule.target_field) {
                                total += val;
                                count += 1.0;
                            }
                        }
                    }
                }
            } else {
                warn!("Invalid glob pattern: {}", search_pattern_str);
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
            } else if let Some(i) = v.as_i64() {
                Some(i as f64)
            } else {
                None
            }
        })
    }
}
