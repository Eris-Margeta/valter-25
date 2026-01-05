use crate::config::{AggregationRule, AggregationLogic};
use glob::glob;
use std::path::Path;
use std::fs;
use serde_yaml::Value;
use tracing::{info, warn, error};
use anyhow::Result;

pub struct Aggregator;

impl Aggregator {
    /// Izračunava vrijednosti za zadana pravila agregacije unutar jednog Islanda (Projekta).
    pub fn calculate(root_path: &Path, rules: &[AggregationRule]) -> Result<std::collections::HashMap<String, f64>> {
        let mut results = std::collections::HashMap::new();

        for rule in rules {
            let mut total = 0.0;
            let mut count = 0.0;

            // Konstruiraj punu putanju za glob (npr. ./DEV/Phoenix/INTERNO/Financije/*.yaml)
            // Pazimo da root_path nema trailing slash, a rule.path ga nema na početku
            let search_pattern = root_path.join(&rule.path);
            let search_pattern_str = search_pattern.to_string_lossy();

            info!("Deep Scan: Searching pattern '{}' for rule '{}'", search_pattern_str, rule.name);

            // Glob traženje datoteka
            if let Ok(paths) = glob(&search_pattern_str) {
                for entry in paths {
                    if let Ok(path) = entry {
                        if path.is_file() {
                            // Parsiraj vrijednost iz fajla
                            if let Some(val) = Self::extract_value(&path, &rule.target_field) {
                                // Ovdje bi se primijenio i 'filter' (npr. status == 'placeno')
                                // Za sada preskačemo filter logiku u ovoj iteraciji radi jednostavnosti,
                                // ali ovdje je mjesto za to.
                                
                                total += val;
                                count += 1.0;
                            }
                        }
                    }
                }
            } else {
                warn!("Invalid glob pattern: {}", search_pattern_str);
            }

            // Izračunaj konačnu vrijednost ovisno o logici
            let final_value = match rule.logic {
                AggregationLogic::Sum => total,
                AggregationLogic::Count => count,
                AggregationLogic::Average => if count > 0.0 { total / count } else { 0.0 },
            };

            results.insert(rule.name.clone(), final_value);
        }

        Ok(results)
    }

    // Pomoćna funkcija za čitanje broja iz YAML-a
    fn extract_value(path: &Path, field: &str) -> Option<f64> {
        let content = fs::read_to_string(path).ok()?;
        let yaml: Value = serde_yaml::from_str(&content).ok()?;

        // Pokušaj naći polje
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

