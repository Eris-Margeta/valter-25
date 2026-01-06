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
use serde_json::json;

pub struct EventProcessor {
    cloud: Arc<SqliteManager>,
    config: Arc<Config>,
}

impl EventProcessor {
    pub fn new(cloud: Arc<SqliteManager>, config: Arc<Config>) -> Self {
        Self { cloud, config }
    }

    pub fn scan_existing_metadata(&self, root_path: &str) {
        // Podržavamo i relativne putanje s '..'
        info!("Scanning existing islands in: {}", root_path);
        for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
            // FIX: Fleksibilnije prepoznavanje meta fajla (podržavamo i naš proprietary)
            if let Some(fname) = entry.file_name().to_str() {
                if fname == "meta.yaml" || fname == "valter.proprietary.yaml" {
                    if let Err(e) = self.process_metadata(entry.path()) {
                        error!("Failed to process {:?}: {}", entry.path(), e);
                    }
                }
            }
        }
    }

    pub async fn handle_event(&self, event: Event) {
        for path in event.paths {
            if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                // FIX: Podrška za oba imena
                if file_name == "meta.yaml" || file_name == "valter.proprietary.yaml" {
                    info!("Metadata Change: {:?}", path);
                    let _ = self.process_metadata(&path);
                } else if path.extension().map_or(false, |ext| ext == "yaml" || ext == "md") {
                    // Trigger deep scan if sub-files change
                    let mut current = path.parent();
                    while let Some(dir) = current {
                        let meta_path_1 = dir.join("meta.yaml");
                        let meta_path_2 = dir.join("valter.proprietary.yaml");
                        
                        if meta_path_1.exists() {
                             info!("Deep scan triggered for project at {:?}", dir);
                             let _ = self.process_metadata(&meta_path_1);
                             break;
                        } else if meta_path_2.exists() {
                             info!("Deep scan triggered for project at {:?}", dir);
                             let _ = self.process_metadata(&meta_path_2);
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

        // Pretpostavljamo prvi definiran Island tip (za sada)
        // U budućnosti bi trebali detektirati tip islanda na temelju sadržaja
        let island_def = &self.config.islands[0]; 

        let project_name = yaml.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Project");

        let project_root = path.parent().unwrap();

        let mut relation_map: HashMap<String, Option<String>> = HashMap::new();
        
        for rel in &island_def.relations {
            if let Some(val_raw) = yaml.get(&rel.field) {
                if let Some(val_str) = val_raw.as_str() {
                    
                    // FIX: Dinamički dohvaćamo ključno polje za Target Cloud
                    // Umjesto hardkodiranog "ime" ili "naziv", tražimo prvo polje u definiciji Clouda
                    let target_cloud_def = self.config.clouds.iter().find(|c| c.name == rel.target_cloud);
                    let key_field = if let Some(def) = target_cloud_def {
                        // Uzmi prvo polje kao identifikator (npr. "handle" ili "name")
                        def.fields.first().map(|f| f.key.as_str()).unwrap_or("id")
                    } else {
                        "id"
                    };

                    let context_info = json!({
                        "source_island_type": island_def.name,
                        "source_island_name": project_name,
                        "field": rel.field
                    }).to_string();

                    match self.cloud.check_or_create_pending(&rel.target_cloud, key_field, val_str, &context_info) {
                        Ok(status) => match status {
                            EntityStatus::Found(uuid) => {
                                relation_map.insert(rel.field.clone(), Some(uuid));
                            },
                            EntityStatus::Pending(_) | EntityStatus::Ambiguous(_, _) => {
                                warn!("Relation '{}' ({}) is PENDING review.", rel.field, val_str);
                                relation_map.insert(rel.field.clone(), None);
                            }
                        },
                        Err(e) => error!("Check error for {}: {}", rel.target_cloud, e)
                    }
                }
            }
        }

        let aggregation_results = Aggregator::calculate(project_root, &island_def.aggregations)?;
        
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

