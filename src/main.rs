mod config;
mod watcher;
mod context_engine;
mod cloud;
mod api;
mod oracle;
mod processor;

use config::Config;
use watcher::Watcher;
use context_engine::ContextEngine;
use cloud::SqliteManager;
use oracle::ToolGenerator;
use processor::EventProcessor;
use anyhow::Result;
use tracing::{info, error};
use tokio::sync::mpsc;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting Strata Daemon...");

    // Load Configuration
    let config_path = "strata.config";
    let config = match Config::load(config_path) {
        Ok(cfg) => {
            info!("Successfully loaded configuration from {}", config_path);
            Arc::new(cfg)
        },
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(e);
        }
    };

    // Initialize Cloud Layer (SQLite)
    let db_path = "strata.db";
    let cloud = Arc::new(SqliteManager::new(db_path)?);
    cloud.init_schema(&config)?;

    // PHASE 5 TEST: Oracle Tool Generation
    info!("--- PHASE 5 TEST: Oracle Tool Generation ---");
    match ToolGenerator::generate_tools(&config) {
        Ok(tools) => {
            let json_out = serde_json::to_string_pretty(&tools)?;
            info!("Generated OpenAI Tools Schema (Length: {} chars)", json_out.len());
        },
        Err(e) => error!("Failed to generate tools: {}", e),
    }

    // Start API Server (Phase 4)
    let cloud_clone = cloud.clone();
    tokio::spawn(async move {
        if let Err(e) = api::start_server(cloud_clone).await {
            error!("API Server failed: {}", e);
        }
    });

    // Initialize Event Processor (Phase 6)
    let processor = EventProcessor::new(cloud.clone(), config.clone());

    // SCAN EXISTING METADATA (Fix for missing data on startup)
    processor.scan_existing_metadata("./DEV");

    // Create channel for file events
    let (tx, mut rx) = mpsc::channel(100);

    // Initialize Watcher on "./DEV"
    let _watcher = Watcher::new("./DEV", tx)?;

    info!("Strata Daemon running. Waiting for file events in ./DEV...");

    // Event Loop
    while let Some(event) = rx.recv().await {
        // Dispatch to Processor
        processor.handle_event(event).await;
    }

    Ok(())
}
