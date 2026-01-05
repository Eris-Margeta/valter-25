use crate::cloud::SqliteManager;
use crate::config::Config;
use crate::aggregator::Aggregator;
use notify::Event;
use std::sync::Arc;
use std::path::Path;
use tracing::{info, error};
use serde_yaml::Value;
use std::fs;
use walkdir::WalkDir;
use std::collections::HashMap;

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
        // Ovdje bi idealno iterirali kroz definicije Islanda u configu
        // Za sada pretpostavljamo da je root_path onaj iz configa (./DEV)
        
        for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_name() == "meta.yaml" {
                if let Err(e) = self.process_metadata(entry.path()) {
                    error!("Failed to process existing metadata {:?}: {}", entry.path(), e);
                }
            }
        }
    }

    pub async fn handle_event(&self, event: Event) {
        for path in event.paths {
            // Ako se promijeni BILO ŠTO u folderu projekta, trebali bi re-skenirati?
            // Za sada, pratimo samo meta.yaml za promjene strukture, 
            // i .yaml fajlove u subfolderima za promjene agregacija.
            
            // Pojednostavljenje: Ako path sadrži "meta.yaml", okini process_metadata.
            // Ako path završava na ".yaml" i nije meta.yaml, moramo naći kojem projektu pripada i re-procesirati ga.
            
            if let Some(file_name) = path.file_name() {
                if file_name == "meta.yaml" {
                    info!("Change detected in meta file: {:?}", path);
                    let _ = self.process_metadata(&path);
                } else if path.extension().map_or(false, |ext| ext == "yaml") {
                    // Netko je dodao račun?
                    // Nađi parent folder koji ima meta.yaml
                    if let Some(parent) = path.parent() {
                        // Ovo je naivno (traži samo direktnog roditelja), trebao bi ići gore dok ne nađe meta.yaml
                        // Ali za prototip, recimo da ako se promijeni bilo što u projektu, 
                        // re-skeniramo cijeli projekt ako znamo gdje je root.
                        // TODO: Implementirati "Find Project Root" logiku.
                        
                        // Za sada, samo logiramo. Prava implementacija zahtijeva mapiranje path -> project_root.
                        info!("Sub-file changed: {:?}. Deep scan trigger pending implementation.", path);
                    }
                }
            }
        }
    }

    fn process_metadata(&self, path: &Path) -> anyhow::Result<()> {
        let content = fs::read_to_string(path)?;
        let yaml: Value = serde_yaml::from_str(&content)?;

        // 1. Identificiraj koji tip Islanda je ovo
        // U configu imamo listu islands. Pretpostavimo prvi za sada (Projekt).
        // U pravom sustavu, meta.yaml bi mogao imati polje "type: Projekt".
        let island_def = &self.config.islands[0]; // Uzimamo "Projekt" definiciju

        // 2. Ekstrahiraj Ime Projekta
        let project_name = yaml.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Project");

        let project_root = path.parent().unwrap();

        // 3. RELACIJE (Dynamic Cloud Linking)
        let mut relation_map = HashMap::new();
        
        for rel in &island_def.relations {
            // rel.field = "klijent" -> tražimo "klijent" u yaml-u
            if let Some(val_raw) = yaml.get(&rel.field) {
                if let Some(val_str) = val_raw.as_str() {
                    // rel.target_cloud = "Klijent" -> upsertamo u tablicu "Klijent"
                    // Ovdje koristimo pretpostavku da je 'naziv' ili 'ime' glavni ključ.
                    // Config bi trebao reći koji je key_field. 
                    // Za TEJL config, Klijent ima 'naziv', Operater ima 'ime'.
                    // Hack: hardcodamo mapiranje za sada ili pogađamo.
                    
                    let key_field = if rel.target_cloud == "Klijent" { "naziv" } else { "ime" };
                    
                    match self.cloud.upsert_entity(&rel.target_cloud, key_field, val_str) {
                        Ok(uuid) => {
                            relation_map.insert(rel.field.clone(), uuid);
                        },
                        Err(e) => error!("Failed to upsert relation {}: {}", rel.target_cloud, e)
                    }
                }
            }
        }

        // 4. AGREGACIJE (Deep Scan)
        let aggregation_results = Aggregator::calculate(project_root, &island_def.aggregations)?;
        
        // 5. SPREMI SVE U ISLAND TABLICU (npr. "Projekt")
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

