mod config;
mod watcher;
mod context_engine;
mod cloud;
mod api;
mod oracle;
mod processor;
mod aggregator;
mod fs_writer; // NOVO

use config::Config;
use watcher::Watcher;
use cloud::SqliteManager;
use oracle::ToolGenerator;
use processor::EventProcessor;
use anyhow::Result;
use tracing::{info, error};
use tokio::sync::mpsc;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    info!("Starting TEJL Strata Daemon v0.4 (Bi-Directional)...");

    let config_path = "strata.config";
    let config = match Config::load(config_path) {
        Ok(cfg) => {
            info!("Loaded Config for: {}", cfg.global.company_name);
            Arc::new(cfg)
        },
        Err(e) => {
            error!("Config Error: {}", e);
            return Err(e);
        }
    };

    let db_path = "strata.db";
    let cloud = Arc::new(SqliteManager::new(db_path)?);
    cloud.init_schema(&config)?;

    // Oracle
    info!("--- Oracle Tools Generation ---");
    if let Ok(tools) = ToolGenerator::generate_tools(&config) {
        let _ = serde_json::to_string(&tools);
    }

    // Start API
    let cloud_clone = cloud.clone();
    tokio::spawn(async move {
        if let Err(e) = api::start_server(cloud_clone).await {
            error!("API Server failed: {}", e);
        }
    });

    let processor = EventProcessor::new(cloud.clone(), config.clone());
    
    // Initial Scan
    processor.scan_existing_metadata("./DEV");

    let (tx, mut rx) = mpsc::channel(100);
    let _watcher = Watcher::new("./DEV", tx)?;

    info!("System Online. Monitoring filesystem...");

    while let Some(event) = rx.recv().await {
        processor.handle_event(event).await;
    }

    Ok(())
}

