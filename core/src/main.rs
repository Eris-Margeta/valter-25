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
use tracing::{info, error}; // Očišćeni importi
use tokio::sync::mpsc;
use std::sync::Arc;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("---------------------------------------------");
    info!("   VALTER ERP - SYSTEM STARTUP v0.1.0");
    info!("   Open Source Edition (MIT)");
    info!("---------------------------------------------");

    let config_candidates = vec!["valter.config", "../valter.config"];
    let mut config_path = "valter.config";
    let mut config_found = false;

    for path in &config_candidates {
        if Path::new(path).exists() {
            config_path = path;
            config_found = true;
            break;
        }
    }

    if !config_found {
        error!("CRITICAL: 'valter.config' not found in current or parent directory.");
        return Err(anyhow::anyhow!("Config missing"));
    }

    let config = match Config::load(config_path) {
        Ok(cfg) => {
            info!("Loaded Configuration from: {}", config_path);
            Arc::new(cfg)
        },
        Err(e) => {
            error!("Config Parse Error: {}", e);
            return Err(e);
        }
    };

    let db_path = if config_path.starts_with("..") { "../valter.db" } else { "valter.db" };
    let cloud = Arc::new(SqliteManager::new(db_path)?);
    cloud.init_schema(&config)?;

    if let Ok(tools) = ToolGenerator::generate_tools(&config) {
        let _ = serde_json::to_string(&tools);
    }

    let cloud_clone = cloud.clone();
    let config_clone_for_api = config.clone(); // Kloniramo za API
    
    // FIX: Prosljeđujemo config u start_server
    tokio::spawn(async move {
        if let Err(e) = api::start_server(cloud_clone, config_clone_for_api).await {
            error!("API Server failed: {}", e);
        }
    });

    let processor = EventProcessor::new(cloud.clone(), config.clone());
    
    let root_scan_path = if config_path.starts_with("..") { ".." } else { "." };
    info!("Scanning workspace for Islands at: '{}'", root_scan_path);
    processor.scan_existing_metadata(root_scan_path);

    let (tx, mut rx) = mpsc::channel(100);
    let _watcher = Watcher::new(root_scan_path, tx)?;

    info!("VALTER is Online. Dashboard at http://localhost:5173");

    while let Some(event) = rx.recv().await {
        processor.handle_event(event).await;
    }

    Ok(())
}

