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
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info, warn};
use watcher::Watcher; // Za provjeru porta

#[derive(Parser)]
#[command(name = "valter")]
#[command(about = "Valter ERP Daemon", long_about = None)]
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

    // --- SETUP PATHS ---
    let user_home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let default_system_home = PathBuf::from(user_home).join(".valter");
    let env_valter_home = env::var("VALTER_HOME").ok().map(PathBuf::from);
    let current_home = env_valter_home
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));

    // PID logic
    let pid_file_path = if let Some(ref path) = env_valter_home {
        path.join("valter.pid")
    } else if current_home.join("valter.dev.config").exists() {
        PathBuf::from("valter.pid")
    } else {
        default_system_home.join("valter.pid")
    };

    // --- COMMAND HANDLING ---
    match cli.command.unwrap_or(Commands::Run) {
        Commands::Stop => {
            return stop_daemon(&pid_file_path);
        }
        Commands::Start => {
            // Aggressive Cleanup
            if is_daemon_running(&pid_file_path) {
                println!("🔄 Found running instance via PID. Stopping...");
                let _ = stop_daemon(&pid_file_path);
                thread::sleep(Duration::from_millis(1000));
            }

            // Production Paths
            let target_home = env_valter_home.unwrap_or(default_system_home);
            fs::create_dir_all(&target_home)?;

            // Check Port Availability BEFORE spawning
            let config_path = target_home.join("valter.config");
            let port = if config_path.exists() {
                if let Ok(c) = fs::read_to_string(&config_path) {
                    if let Ok(cfg) = serde_yaml::from_str::<Config>(&c) {
                        cfg.global.port
                    } else {
                        9090
                    }
                } else {
                    9090
                }
            } else {
                9090
            };

            if is_port_in_use(port) {
                eprintln!("❌ Port {} is ALREADY in use by another process!", port);
                eprintln!("   Please identify and kill the zombie process:");
                eprintln!("   lsof -ti:{} | xargs kill -9", port);
                process::exit(1);
            }

            // Logs
            let log_path = target_home.join("valter.log");
            let log_file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path)?;

            let exe = env::current_exe()?;

            println!("🚀 Starting Valter Daemon...");
            println!("   Home: {:?}", target_home);
            println!("   Logs: {:?}", log_path);
            println!("   URL:  http://localhost:{}", port);

            process::Command::new(exe)
                .arg("run")
                .env("VALTER_HOME", &target_home)
                .stdout(log_file.try_clone()?)
                .stderr(log_file)
                .spawn()?;

            println!("✅ Started successfully.");
            return Ok(());
        }
        Commands::Run => {
            // Logging Configuration (Human Readable Time)
            // If in system mode (ENV set), no ANSI. If dev, yes ANSI.
            let use_ansi = env::var("VALTER_HOME").is_err();

            // Custom formatter for "2026-01-06 16:30:00"
            let timer =
                tracing_subscriber::fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string());

            tracing_subscriber::fmt()
                .with_ansi(use_ansi)
                .with_timer(timer)
                .init();

            let pid = process::id();
            let _ = fs::write(&pid_file_path, pid.to_string());
            info!("PID: {} | Log Path: {:?}", pid, pid_file_path);
        }
    }

    // =========================================================
    // THE HOT RELOAD LOOP
    // =========================================================

    // Setup paths only once
    let valter_home = env::var("VALTER_HOME").unwrap_or_else(|_| ".".to_string());
    let home_path = PathBuf::from(&valter_home);
    let is_system_mode = valter_home != ".";

    loop {
        info!("🔁 System Loop: Initializing...");

        // 1. Config Load
        let config_path = if is_system_mode && home_path.join("valter.config").exists() {
            home_path.join("valter.config")
        } else if PathBuf::from("valter.dev.config").exists() {
            PathBuf::from("valter.dev.config")
        } else if PathBuf::from("../valter.dev.config").exists() {
            PathBuf::from("../valter.dev.config")
        } else if PathBuf::from("valter.config").exists() {
            PathBuf::from("valter.config")
        } else {
            error!("CRITICAL: Config not found. Retrying in 5s...");
            thread::sleep(Duration::from_secs(5));
            continue;
        };

        let config = match Config::load(config_path.to_str().unwrap()) {
            Ok(c) => Arc::new(c),
            Err(e) => {
                error!("Config Syntax Error: {}. Retrying in 5s...", e);
                thread::sleep(Duration::from_secs(5));
                continue;
            }
        };

        // 2. Database
        let db_path = if is_system_mode {
            home_path.join("valter.db")
        } else {
            PathBuf::from("valter.db")
        };
        let cloud = match SqliteManager::new(db_path.to_str().unwrap()) {
            Ok(c) => Arc::new(c),
            Err(e) => {
                error!("DB Error: {}", e);
                return Err(e);
            }
        };
        if let Err(e) = cloud.init_schema(&config) {
            error!("Schema Error: {}", e);
        }

        // 3. Oracle
        if let Ok(tools) = ToolGenerator::generate_tools(&config) {
            let _ = serde_json::to_string(&tools);
        }

        // 4. CHANNELS for Shutdown
        let (shutdown_tx, _) = broadcast::channel(1);
        let (fs_tx, mut fs_rx) = mpsc::channel(100);

        // 5. API Task
        let cloud_clone = cloud.clone();
        let config_clone = config.clone();
        let api_rx = shutdown_tx.subscribe();

        let api_handle = tokio::spawn(async move {
            if let Err(e) = api::start_server(cloud_clone, config_clone, api_rx).await {
                error!("API Fatal: {}", e);
                process::exit(1);
            }
        });

        // 6. Processor & Watcher
        let processor = EventProcessor::new(cloud.clone(), config.clone());
        let watch_root = if is_system_mode {
            valter_home.clone()
        } else {
            ".".to_string()
        };

        info!("👁️  Watching: {}", watch_root);
        processor.scan_existing_metadata(&watch_root);

        // Watcher runs in separate thread but sends to fs_rx
        // We need to keep the watcher alive
        let _watcher = match Watcher::new(&watch_root, fs_tx) {
            Ok(w) => w,
            Err(e) => {
                error!("Watcher failed: {}", e);
                return Err(e);
            }
        };

        info!("✅ System Operational.");

        // 7. Event Loop (Blocks until Reload or Shutdown)
        let mut reload_requested = false;

        loop {
            tokio::select! {
                Some(event) = fs_rx.recv() => {
                    // Check if config changed
                    for path in &event.paths {
                        if path.ends_with("valter.config") || path.ends_with("valter.dev.config") {
                            info!("♻️  Configuration Changed! Reloading system...");
                            reload_requested = true;
                            break;
                        }
                    }
                    if reload_requested { break; }

                    // Normal processing
                    processor.handle_event(event).await;
                }
                // Handle Ctrl+C if in foreground
                _ = tokio::signal::ctrl_c() => {
                    info!("🛑 Shutdown signal received.");
                    let _ = shutdown_tx.send(());
                    let _ = fs::remove_file(&pid_file_path);
                    return Ok(());
                }
            }
        }

        // 8. Reload Logic
        if reload_requested {
            // Signal API to stop
            let _ = shutdown_tx.send(());
            // Wait for API to finish
            let _ = api_handle.await;
            info!("--- RELOADING CORE ---");
            // Loop restarts, creating new Cloud, new Schema, new API on same port.
        }
    }
}

// Helpers

fn is_port_in_use(port: u16) -> bool {
    TcpListener::bind(("0.0.0.0", port)).is_err()
}

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
        // Wait loop
        for _ in 0..10 {
            if signal::kill(Pid::from_raw(pid), None).is_err() {
                break;
            }
            thread::sleep(Duration::from_millis(500));
        }
        println!("Stopped.");
        let _ = fs::remove_file(pid_path);
    } else {
        println!("Failed/Already dead.");
        let _ = fs::remove_file(pid_path);
    }
    Ok(())
}
