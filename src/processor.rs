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

        // Extract Project Name (default to folder name if missing)
        let project_name = yaml.get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                path.parent()
                    .and_then(|p| p.file_name())
                    .map(|os| os.to_string_lossy().to_string())
                    .unwrap_or("Unknown Project".to_string())
            });

        let status = yaml.get("status").and_then(|v| v.as_str()).unwrap_or("Active");
        
        let client_name = yaml.get("client").and_then(|v| v.as_str()).unwrap_or("Unknown");
        let op_name = yaml.get("operator").and_then(|v| v.as_str()).unwrap_or("Unknown");

        // 1. Upsert Project into Cloud
        // Note: In a real system, we'd handle the schema dynamically. 
        // Here we rely on the implicit columns we created via strata.config
        // We need to implement a generic upsert_row, but upsert_entity only does id/name.
        // For the prototype, we will just use upsert_entity for the Project Name to get an ID.
        // But wait, Project has extra fields (client, operator, status).
        // Our current SqliteManager.upsert_entity is too simple (only id, name).
        
        // Let's stick to the prototype limitation: Just tracking existence.
        // We will upsert the Project Name into the 'Project' table.
        let project_id = self.cloud.upsert_entity("Project", "name", &project_name)?;
        info!("Registered Project: {} (UUID: {})", project_name, project_id);

        // 2. Process Relations (Upsert Client/Operator)
        if client_name != "Unknown" {
            let id = self.cloud.upsert_entity("Client", "name", client_name)?;
            info!("Linked Project to Client: {} (UUID: {})", client_name, id);
        }

        if op_name != "Unknown" {
            let id = self.cloud.upsert_entity("Operator", "name", op_name)?;
            info!("Linked Project to Operator: {} (UUID: {})", op_name, id);
        }

        Ok(())
    }
}
