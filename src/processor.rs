use crate::cloud::{SqliteManager, EntityStatus};
use crate::config::Config;
use crate::aggregator::Aggregator;
use notify::Event;
use std::sync::Arc;
use std::path::Path;
use tracing::{info, error, warn};
use serde_yaml::Value;
use std::fs;
use std::collections::HashMap;
use walkdir::WalkDir;

pub struct EventProcessor {
    cloud: Arc<SqliteManager>,
    config: Arc<Config>,
}

impl EventProcessor {
    pub fn new(cloud: Arc<SqliteManager>, config: Arc<Config>) -> Self {
        Self { cloud, config }
    }

    pub fn scan_existing_metadata(&self, root_path: &str) {
        info!("Scanning existing islands in: {}", root_path);
        for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_name() == "meta.yaml" {
                if let Err(e) = self.process_metadata(entry.path()) {
                    error!("Failed to process {:?}: {}", entry.path(), e);
                }
            }
        }
    }

    pub async fn handle_event(&self, event: Event) {
        for path in event.paths {
            if let Some(file_name) = path.file_name() {
                if file_name == "meta.yaml" {
                    info!("Metadata Change: {:?}", path);
                    let _ = self.process_metadata(&path);
                } else if path.extension().map_or(false, |ext| ext == "yaml") {
                    // Ako se promijeni pod-fajl (račun), re-skeniraj projekt.
                    // Tražimo meta.yaml u parent direktorijima.
                    let mut current = path.parent();
                    while let Some(dir) = current {
                        let meta_path = dir.join("meta.yaml");
                        if meta_path.exists() {
                             info!("Deep scan triggered for project at {:?}", dir);
                             let _ = self.process_metadata(&meta_path);
                             break;
                        }
                        current = dir.parent();
                    }
                }
            }
        }
    }

    fn process_metadata(&self, path: &Path) -> anyhow::Result<()> {
        let content = fs::read_to_string(path)?;
        let yaml: Value = serde_yaml::from_str(&content)?;

        // Pretpostavljamo prvi tip islanda iz configa (Projekt)
        let island_def = &self.config.islands[0]; 

        let project_name = yaml.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Project");

        let project_root = path.parent().unwrap();

        // --- RELACIJE s provjerom (Safety Valve) ---
        let mut relation_map: HashMap<String, Option<String>> = HashMap::new();
        
        for rel in &island_def.relations {
            if let Some(val_raw) = yaml.get(&rel.field) {
                if let Some(val_str) = val_raw.as_str() {
                    // Određivanje ključnog polja (npr. Operater -> ime, Klijent -> naziv)
                    // Ovo bi trebalo biti dio Configa, ali za sad heuristika:
                    // Ako Cloud ima polje 'naziv', koristi to. Ako ima 'ime', koristi to.
                    // Za TEJL config znamo točno.
                    let key_field = if rel.target_cloud == "Operater" { "ime" } else { "naziv" };
                    let context_info = format!("Pronađeno u projektu: {}", project_name);

                    // PROVJERA (Check or Pending)
                    match self.cloud.check_or_create_pending(&rel.target_cloud, key_field, val_str, &context_info) {
                        Ok(status) => match status {
                            EntityStatus::Found(uuid) => {
                                relation_map.insert(rel.field.clone(), Some(uuid));
                            },
                            EntityStatus::Pending(_) | EntityStatus::Ambiguous(_, _) => {
                                warn!("Relation '{}' ({}) is PENDING review. Not linking yet.", rel.field, val_str);
                                // Ostavljamo NULL u bazi. Frontend će prikazati upozorenje.
                                relation_map.insert(rel.field.clone(), None);
                            }
                        },
                        Err(e) => error!("Check error for {}: {}", rel.target_cloud, e)
                    }
                }
            }
        }

        // --- AGREGACIJE (Deep Scan) ---
        let aggregation_results = Aggregator::calculate(project_root, &island_def.aggregations)?;
        
        // --- SPREMANJE (Upsert Island) ---
        self.cloud.upsert_island(
            &island_def.name,
            project_name,
            project_root.to_string_lossy().as_ref(),
            &relation_map,
            &aggregation_results
        )?;

        Ok(())
    }
}
