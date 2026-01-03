use crate::cloud::SqliteManager;
use crate::config::Config;
use notify::Event;
use std::sync::Arc;
use std::path::Path;
use tracing::{info, error, warn};
use serde_yaml::Value;
use std::fs;
use walkdir::WalkDir;

pub struct EventProcessor {
    cloud: Arc<SqliteManager>,
    #[allow(dead_code)]
    config: Arc<Config>,
}

impl EventProcessor {
    pub fn new(cloud: Arc<SqliteManager>, config: Arc<Config>) -> Self {
        Self { cloud, config }
    }

    pub fn scan_existing_metadata(&self, root_path: &str) {
        info!("Scanning existing metadata in: {}", root_path);
        for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_name() == "meta.yaml" {
                info!("Found existing metadata: {:?}", entry.path());
                if let Err(e) = self.process_metadata(entry.path()) {
                    error!("Failed to process existing metadata {:?}: {}", entry.path(), e);
                }
            }
        }
    }

    pub async fn handle_event(&self, event: Event) {
        for path in event.paths {
            // We only care about meta.yaml files for now (The Island Root)
            if let Some(file_name) = path.file_name() {
                if file_name == "meta.yaml" {
                    info!("Processing Metadata Change: {:?}", path);
                    if let Err(e) = self.process_metadata(&path) {
                        error!("Failed to process metadata {:?}: {}", path, e);
                    }
                }
            }
        }
    }

    fn process_metadata(&self, path: &Path) -> anyhow::Result<()> {
        let content = fs::read_to_string(path)?;
        let yaml: Value = serde_yaml::from_str(&content)?;

        // HARDCODED LOGIC FOR PROTOTYPE (In real version, this comes from Config.relations)
        // We look for 'client' and 'operator' fields.
        
        if let Some(client_val) = yaml.get("client") {
            if let Some(client_name) = client_val.as_str() {
                // Upsert Client
                let id = self.cloud.upsert_entity("Client", "name", client_name)?;
                info!("Linked Project to Client: {} (UUID: {})", client_name, id);
            }
        }

        if let Some(op_val) = yaml.get("operator") {
            if let Some(op_name) = op_val.as_str() {
                // Upsert Operator
                let id = self.cloud.upsert_entity("Operator", "name", op_name)?;
                info!("Linked Project to Operator: {} (UUID: {})", op_name, id);
            }
        }

        Ok(())
    }
}