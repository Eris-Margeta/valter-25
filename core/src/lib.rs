// core/src/lib.rs

pub mod aggregator;
pub mod api;
pub mod cloud;
pub mod config;
pub mod context_engine;
pub mod fs_writer;
pub mod oracle;
pub mod processor;
pub mod watcher;

use anyhow::Result;
use cloud::SqliteManager;
use config::{env::EnvConfig, Config};
use oracle::ToolGenerator;
use processor::EventProcessor;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info};
use watcher::Watcher;

pub async fn run(valter_home: PathBuf, is_dev_mode: bool) -> Result<()> {
    let pid_file = valter_home.join("valter.pid");

    // Ensure the home directory exists
    if !valter_home.exists() {
        fs::create_dir_all(&valter_home)?;
    }

    let _ = fs::write(&pid_file, process::id().to_string());
    info!(
        "VALTER ONLINE. Home: {:?}, DevMode: {}",
        valter_home, is_dev_mode
    );

    let env_config = Arc::new(EnvConfig::init());
    info!("Environment Config Status: {:?}", env_config.status);

    loop {
        let config_path = if is_dev_mode {
            valter_home.join("valter.dev.config")
        } else {
            valter_home.join("valter.config")
        };

        if !config_path.exists() {
            error!("Config missing at {:?}. Sleeping...", config_path);
            thread::sleep(Duration::from_secs(5));
            continue;
        }

        let config = match Config::load(config_path.to_str().unwrap()) {
            Ok(c) => Arc::new(c),
            Err(e) => {
                error!("Config Error: {}", e);
                thread::sleep(Duration::from_secs(5));
                continue;
            }
        };

        let db_path = valter_home.join("valter.db");
        let cloud: Arc<SqliteManager> = match SqliteManager::new(db_path.to_str().unwrap()) {
            Ok(c) => Arc::new(c),
            Err(e) => {
                error!("DB Error: {}", e);
                // ISPRAVAK: Uklonjen nepotreban `.into()`
                return Err(e);
            }
        };
        if let Err(e) = cloud.init_schema(&config) {
            error!("Schema init error: {}", e);
        }
        let _ = ToolGenerator::generate_tools(&config);

        let (shutdown_tx, _) = broadcast::channel(1);
        let (fs_tx, mut fs_rx) = mpsc::channel(100);

        let processor = Arc::new(EventProcessor::new(cloud.clone(), config.clone()));
        processor.scan_on_startup();

        let cloud_clone = cloud.clone();
        let config_clone = config.clone();
        let processor_clone = processor.clone();
        let env_config_clone = env_config.clone();
        let api_rx = shutdown_tx.subscribe();
        let api_handle = tokio::spawn(async move {
            if let Err(e) = api::start_server(
                cloud_clone,
                config_clone,
                processor_clone,
                env_config_clone,
                api_rx,
            )
            .await
            {
                error!("API Fatal: {}", e);
                process::exit(1);
            }
        });

        let mut watch_paths = Vec::new();
        if let Some(p) = config_path.parent() {
            watch_paths.push(p.to_string_lossy().to_string());
        }
        for island in &config.islands {
            let clean = island.root_path.replace('*', "");
            watch_paths.push(clean);
        }
        watch_paths.dedup();

        let _watcher = Watcher::new(watch_paths, fs_tx)?;
        info!("System Operational. Waiting for events...");

        let mut reload = false;
        loop {
            tokio::select! {
                Some(event) = fs_rx.recv() => {
                    for p in &event.paths {
                        if p.to_string_lossy().contains("valter.dev.config") || p.to_string_lossy().contains("valter.config") {
                            info!("Config changed. Reloading...");
                            reload = true; break;
                        }
                    }
                    if reload { break; }
                    processor.handle_event(event).await;
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Ctrl-C received. Shutting down...");
                    let _ = shutdown_tx.send(());
                    let _ = fs::remove_file(&pid_file);
                    return Ok(());
                }
            }
        }

        if reload {
            info!("Shutting down services for reload...");
            let _ = shutdown_tx.send(());
            let _ = api_handle.await;
            info!("Reloading main loop.");
        }
    }
}
