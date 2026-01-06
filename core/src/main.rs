mod aggregator;
mod api;
mod cloud;
mod config;
mod context_engine;
mod fs_writer;
mod oracle;
mod processor;
mod watcher;

use anyhow::Result;
use clap::{Parser, Subcommand};
use cloud::SqliteManager;
use config::Config;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use oracle::ToolGenerator;
use processor::EventProcessor;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info, warn};
use watcher::Watcher;

#[derive(Parser)]
#[command(name = "valter")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Start,
    Stop,
    Run,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let cli = Cli::parse();

    // 1. DETERMINE MODE
    let env_home = env::var("VALTER_HOME").ok().map(PathBuf::from);
    let user_home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or(".".into());
    let default_prod_home = PathBuf::from(&user_home).join(".valter");

    // Logic: If VALTER_HOME env var is set, use it.
    // If NOT set, check if we are in a dev environment (Justfile usually sets VALTER_HOME to pwd)
    // If running binary without vars, default to ~/.valter

    let (valter_home, is_dev_mode) = if let Some(h) = env_home {
        (h, true) // Assuming manually set HOME means we control it (or Justfile dev)
    } else {
        (default_prod_home.clone(), false)
    };

    let pid_file = valter_home.join("valter.pid");

    match cli.command.unwrap_or(Commands::Run) {
        Commands::Stop => return stop_daemon(&pid_file),
        Commands::Start => {
            if is_daemon_running(&pid_file) {
                println!("Stopping running instance...");
                let _ = stop_daemon(&pid_file);
                thread::sleep(Duration::from_millis(1000));
            }
            fs::create_dir_all(&valter_home)?;
            let log_file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(valter_home.join("valter.log"))?;
            let exe = env::current_exe()?;

            process::Command::new(exe)
                .arg("run")
                .env("VALTER_HOME", &valter_home)
                .stdout(log_file.try_clone()?)
                .stderr(log_file)
                .spawn()?;

            println!("✅ Valter Started. Home: {:?}", valter_home);
            return Ok(());
        }
        Commands::Run => {
            // Init logging
            let timer =
                tracing_subscriber::fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string());
            tracing_subscriber::fmt().with_timer(timer).init();
            let _ = fs::write(&pid_file, process::id().to_string());
            info!(
                "VALTER ONLINE. Home: {:?}, DevMode: {}",
                valter_home, is_dev_mode
            );
        }
    }

    loop {
        // 2. CONFIG LOAD
        let config_path = if is_dev_mode {
            // In dev mode (Justfile), we are at repo root. Look for dev config.
            valter_home.join("valter.dev.config")
        } else {
            // In prod mode, strict look at ~/.valter/valter.config
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

        // 3. DB & SCHEMA
        let db_path = valter_home.join("valter.db");
        let cloud: Arc<SqliteManager> = match SqliteManager::new(db_path.to_str().unwrap()) {
            Ok(c) => Arc::new(c),
            Err(e) => {
                error!("DB Error: {}", e);
                return Err(e);
            }
        };
        if let Err(e) = cloud.init_schema(&config) {
            error!("Schema: {}", e);
        }
        let _ = ToolGenerator::generate_tools(&config);

        // 4. INFRASTRUCTURE
        let (shutdown_tx, _) = broadcast::channel(1);
        let (fs_tx, mut fs_rx) = mpsc::channel(100);

        // 5. API
        let cloud_clone = cloud.clone();
        let config_clone = config.clone();
        let api_rx = shutdown_tx.subscribe();
        let api_handle = tokio::spawn(async move {
            if let Err(e) = api::start_server(cloud_clone, config_clone, api_rx).await {
                error!("API Fatal: {}", e);
                process::exit(1);
            }
        });

        // 6. PROCESSOR & SCAN
        let processor = EventProcessor::new(cloud.clone(), config.clone());
        // Clean & Scan on startup to prevent ghost data
        processor.scan_on_startup();

        // 7. WATCHER
        let mut watch_paths = Vec::new();
        // Config file folder
        if let Some(p) = config_path.parent() {
            watch_paths.push(p.to_string_lossy().to_string());
        }

        // Island Roots
        for island in &config.islands {
            let clean = island.root_path.replace("*", "");
            // Resolve relative paths in dev mode
            if is_dev_mode && clean.starts_with("./") {
                if let Ok(abs) = fs::canonicalize(&clean) {
                    watch_paths.push(abs.to_string_lossy().to_string());
                } else {
                    watch_paths.push(clean);
                }
            } else {
                watch_paths.push(clean);
            }
        }

        let _watcher = Watcher::new(watch_paths, fs_tx)?;
        info!("System Operational.");

        // 8. EVENT LOOP
        let mut reload = false;
        loop {
            tokio::select! {
                Some(event) = fs_rx.recv() => {
                    for p in &event.paths {
                        if p.to_string_lossy().contains("valter.config") {
                            info!("Config changed. Reloading...");
                            reload = true; break;
                        }
                    }
                    if reload { break; }
                    processor.handle_event(event).await;
                }
                _ = tokio::signal::ctrl_c() => {
                    let _ = shutdown_tx.send(());
                    let _ = fs::remove_file(&pid_file);
                    return Ok(());
                }
            }
        }

        if reload {
            let _ = shutdown_tx.send(());
            let _ = api_handle.await;
        }
    }
}

// ... helpers stop_daemon/is_daemon_running same as before
fn is_daemon_running(pid_path: &Path) -> bool {
    if !pid_path.exists() {
        return false;
    }
    if let Ok(c) = fs::read_to_string(pid_path) {
        if let Ok(pid) = c.trim().parse::<i32>() {
            if signal::kill(Pid::from_raw(pid), None).is_ok() {
                return true;
            }
        }
    }
    let _ = fs::remove_file(pid_path);
    false
}

fn stop_daemon(pid_path: &Path) -> Result<()> {
    if !pid_path.exists() {
        println!("Valter is not running.");
        return Ok(());
    }
    let pid = fs::read_to_string(pid_path)?.trim().parse::<i32>()?;
    println!("Stopping PID {}...", pid);
    if signal::kill(Pid::from_raw(pid), Signal::SIGTERM).is_ok() {
        thread::sleep(Duration::from_millis(500));
        println!("Stopped.");
    }
    let _ = fs::remove_file(pid_path);
    Ok(())
}
