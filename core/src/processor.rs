use crate::aggregator::Aggregator;
use crate::cloud::{EntityStatus, SqliteManager};
use crate::config::{Config, IslandDefinition};
use notify::Event;
use serde_json::json;
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use walkdir::WalkDir;

pub struct EventProcessor {
    cloud: Arc<SqliteManager>,
    config: Arc<Config>,
}

impl EventProcessor {
    pub fn new(cloud: Arc<SqliteManager>, config: Arc<Config>) -> Self {
        Self { cloud, config }
    }

    /// Skenira sve definirane lokacije iz Configa prilikom pokretanja
    pub fn scan_on_startup(&self) {
        info!("🔍 Initial Scan: Starting...");
        for island_def in &self.config.islands {
            // Uklanjamo wildcard (*) da dobijemo base path
            let base_path_str = island_def.root_path.replace("*", "");
            let base_path = Path::new(&base_path_str);

            if !base_path.exists() {
                warn!(
                    "⚠️  Path not found: {:?} (Skipping scan for '{}')",
                    base_path, island_def.name
                );
                continue;
            }

            info!(
                "📂 Scanning Island Type '{}' in: {:?}",
                island_def.name, base_path
            );

            for entry in WalkDir::new(base_path).into_iter().filter_map(|e| e.ok()) {
                if let Some(fname) = entry.file_name().to_str() {
                    // Provjeravamo odgovara li ime fajla definiciji (npr. meta.yaml)
                    if fname == island_def.meta_file {
                        if let Err(e) = self.process_metadata(entry.path(), island_def) {
                            error!("❌ Failed to process {:?}: {}", entry.path(), e);
                        }
                    }
                }
            }
        }
        info!("✅ Initial Scan Complete.");
    }

    pub async fn handle_event(&self, event: Event) {
        for path in event.paths {
            // 1. Pokušaj naći Island Definiciju koja odgovara ovom fajlu
            if let Some(island_def) = self.find_matching_island_def(&path) {
                info!(
                    "⚡ Metadata Change Detected: {:?} (Type: {})",
                    path, island_def.name
                );
                let _ = self.process_metadata(&path, island_def);
            }
            // 2. Ako nije meta fajl, možda je sub-file (retrigger deep scan)
            else if path
                .extension()
                .map_or(false, |ext| ext == "yaml" || ext == "md" || ext == "txt")
            {
                // Penjemo se gore dok ne nađemo meta fajl koji definira Island
                let mut current = path.parent();
                while let Some(dir) = current {
                    // Provjeri ima li ovaj folder ikakav meta fajl definiran u configu
                    if let Some(parent_def) = self.find_active_meta_in_dir(dir) {
                        info!("🔄 Deep scan triggered by sub-file change in {:?}", dir);
                        let meta_path = dir.join(&parent_def.meta_file);
                        let _ = self.process_metadata(&meta_path, parent_def);
                        break;
                    }
                    current = dir.parent();
                }
            }
        }
    }

    /// Pomoćna funkcija: Nalazi definiciju na temelju imena fajla i putanje
    fn find_matching_island_def<'a>(&'a self, path: &Path) -> Option<&'a IslandDefinition> {
        let file_name = path.file_name()?.to_str()?;

        for island in &self.config.islands {
            if island.meta_file == file_name {
                // Provjera putanje (Jako bitno da ne miješamo tipove ako imaju isto ime meta fajla)
                // Jednostavna provjera: Da li putanja fajla počinje s root pathom islanda?
                // Moramo maknuti glob charove.
                let root_clean = island.root_path.replace("*", "").replace("./", "");
                // Oprez: Canonicalization bi bilo idealno, ali za sada string match:
                let path_str = path.to_string_lossy();

                // Hack za dev environment gdje su pathovi relativni vs apsolutni
                if path_str.contains(&root_clean) {
                    return Some(island);
                }
            }
        }
        None
    }

    /// Pomoćna funkcija: Provjerava postoji li validan meta fajl u direktoriju
    fn find_active_meta_in_dir<'a>(&'a self, dir: &Path) -> Option<&'a IslandDefinition> {
        for island in &self.config.islands {
            let candidate = dir.join(&island.meta_file);
            if candidate.exists() {
                return Some(island);
            }
        }
        None
    }

    fn process_metadata(&self, path: &Path, island_def: &IslandDefinition) -> anyhow::Result<()> {
        debug!("Processing: {:?}", path);
        let content = fs::read_to_string(path)?;
        let yaml: Value = serde_yaml::from_str(&content)?;

        let project_name = yaml.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown Project");

        let project_root = path.parent().unwrap();
        let mut relation_map: HashMap<String, Option<String>> = HashMap::new();

        // RELATIONS LOGIC
        for rel in &island_def.relations {
            if let Some(val_raw) = yaml.get(&rel.field) {
                if let Some(val_str) = val_raw.as_str() {
                    // Dinamičko traženje ID polja u target cloudu
                    let target_cloud_def =
                        self.config.clouds.iter().find(|c| c.name == rel.target_cloud);
                    let key_field = target_cloud_def
                        .and_then(|def| def.fields.first())
                        .map(|f| f.key.as_str())
                        .unwrap_or("id");

                    let context_info = json!({
                        "source_island_type": island_def.name,
                        "source_island_name": project_name,
                        "field": rel.field
                    })
                    .to_string();

                    match self.cloud.check_or_create_pending(
                        &rel.target_cloud,
                        key_field,
                        val_str,
                        &context_info,
                    ) {
                        Ok(status) => match status {
                            EntityStatus::Found(uuid) => {
                                relation_map.insert(rel.field.clone(), Some(uuid));
                            }
                            EntityStatus::Pending(_) | EntityStatus::Ambiguous(_, _) => {
                                warn!("Relation '{}' ({}) is PENDING review.", rel.field, val_str);
                                relation_map.insert(rel.field.clone(), None);
                            }
                        },
                        Err(e) => error!("Check error for {}: {}", rel.target_cloud, e),
                    }
                }
            }
        }

        // AGGREGATION LOGIC
        let aggregation_results = Aggregator::calculate(project_root, &island_def.aggregations)?;

        // UPSERT
        self.cloud.upsert_island(
            &island_def.name,
            project_name,
            project_root.to_string_lossy().as_ref(),
            &relation_map,
            &aggregation_results,
        )?;

        Ok(())
    }
}
