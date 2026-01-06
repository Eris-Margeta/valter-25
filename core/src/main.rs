mod config;
mod watcher;
mod context_engine;
mod cloud;
mod api;
mod oracle;
mod processor;
mod aggregator;
mod fs_writer;

use config::Config;
use watcher::Watcher;
use cloud::SqliteManager;
use oracle::ToolGenerator;
use processor::EventProcessor;
use anyhow::Result;
use tracing::{info, error};
use tokio::sync::mpsc;
use std::sync::Arc;
use std::path::PathBuf;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("---------------------------------------------");
    info!("   VALTER ERP - SYSTEM STARTUP v0.1.0");
    info!("   Open Source Edition (MIT)");
    info!("---------------------------------------------");

    let valter_home = env::var("VALTER_HOME").unwrap_or_else(|_| ".".to_string());
    let home_path = PathBuf::from(&valter_home);
    let is_system_mode = valter_home != ".";

    if is_system_mode {
        info!("Mode: SYSTEM (Home: {:?})", home_path);
    } else {
        info!("Mode: DEVELOPMENT / LOCAL");
    }

    // PRIORITY LIST FOR CONFIG:
    // 1. System Mode: ~/.valter/valter.config
    // 2. Dev Mode: ./valter.dev.config (Repo Root)
    // 3. Dev Mode Fallback: ../valter.dev.config (If running from core/)
    // 4. Legacy/Local: valter.config

    let config_path = if is_system_mode && home_path.join("valter.config").exists() {
        home_path.join("valter.config")
    } else if PathBuf::from("valter.dev.config").exists() {
        // Root run with dev config
        PathBuf::from("valter.dev.config")
    } else if PathBuf::from("../valter.dev.config").exists() {
        // Core run with dev config in parent
        PathBuf::from("../valter.dev.config")
    } else if PathBuf::from("valter.config").exists() {
        // Fallback
        PathBuf::from("valter.config")
    } else if PathBuf::from("../valter.config").exists() {
        // Fallback parent
        PathBuf::from("../valter.config")
    } else {
        error!("CRITICAL: Configuration file not found.");
        error!("Expected 'valter.dev.config' (Dev) or 'valter.config' (System).");
        return Err(anyhow::anyhow!("Config missing"));
    };

    let config = match Config::load(config_path.to_str().unwrap()) {
        Ok(cfg) => {
            info!("Loaded Configuration from: {:?}", config_path);
            Arc::new(cfg)
        },
        Err(e) => {
            error!("Config Parse Error: {}", e);
            return Err(e);
        }
    };

    // Database Location logic
    let db_path = if is_system_mode {
        home_path.join("valter.db")
    } else if config_path.starts_with("..") {
        PathBuf::from("../valter.db")
    } else {
        PathBuf::from("valter.db")
    };
    
    let cloud = Arc::new(SqliteManager::new(db_path.to_str().unwrap())?);
    cloud.init_schema(&config)?;

    if let Ok(tools) = ToolGenerator::generate_tools(&config) {
        let _ = serde_json::to_string(&tools);
    }

    let cloud_clone = cloud.clone();
    let config_clone = config.clone();
    tokio::spawn(async move {
        if let Err(e) = api::start_server(cloud_clone, config_clone).await {
            error!("API Server failed: {}", e);
        }
    });

    let processor = EventProcessor::new(cloud.clone(), config.clone());
    
    // Scan Root Logic:
    // If we loaded valter.dev.config from root, we are likely in root.
    // We should scan the path defined in config (handled by processor), 
    // BUT Watcher needs a root to watch.
    // If config says "./dev-projects-folder", watcher should probably watch "." to catch that folder.
    
    let watch_root = if is_system_mode {
        valter_home.clone()
    } else if config_path.starts_with("..") {
        "..".to_string()
    } else {
        ".".to_string()
    };

    info!("Watcher attached to: '{}'", watch_root);
    processor.scan_existing_metadata(&watch_root);

    let (tx, mut rx) = mpsc::channel(100);
    let _watcher = Watcher::new(&watch_root, tx)?;

    info!("VALTER is Online. Dashboard at http://localhost:5173");

    while let Some(event) = rx.recv().await {
        processor.handle_event(event).await;
    }

    Ok(())
}

