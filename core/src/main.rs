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
use tokio::sync::mpsc;
use tracing::{error, info};
use watcher::Watcher;

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

    let user_home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let default_system_home = PathBuf::from(user_home).join(".valter");

    let env_valter_home = env::var("VALTER_HOME").ok().map(PathBuf::from);
    let current_home = env_valter_home
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));

    // Provjera moda za logging
    let is_system_mode_logging = env_valter_home.is_some();

    let pid_file_path = if let Some(ref path) = env_valter_home {
        path.join("valter.pid")
    } else if current_home.join("valter.dev.config").exists() {
        PathBuf::from("valter.pid")
    } else {
        default_system_home.join("valter.pid")
    };

    match cli.command.unwrap_or(Commands::Run) {
        Commands::Stop => {
            return stop_daemon(&pid_file_path);
        }
        Commands::Start => {
            if is_daemon_running(&pid_file_path) {
                println!("🔄 Found running instance. Stopping it first...");
                let _ = stop_daemon(&pid_file_path);
                thread::sleep(Duration::from_millis(1000));
            }

            println!("🚀 Initializing Valter Daemon...");

            let target_home = env_valter_home.unwrap_or(default_system_home);
            fs::create_dir_all(&target_home)?;

            let log_path = target_home.join("valter.log");
            // Otvaramo log file
            let log_file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path)?;

            let config_path = target_home.join("valter.config");
            let port = if config_path.exists() {
                if let Ok(content) = fs::read_to_string(&config_path) {
                    if let Ok(cfg) = serde_yaml::from_str::<Config>(&content) {
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

            let exe = env::current_exe()?;

            println!("   Home: {:?}", target_home);
            println!("   Logs: {:?}", log_path);
            println!("   URL:  http://localhost:{}", port);

            process::Command::new(exe)
                .arg("run")
                .env("VALTER_HOME", &target_home)
                .stdout(log_file.try_clone()?)
                .stderr(log_file)
                .spawn()?;

            println!("✅ Valter started in background.");
            return Ok(());
        }
        Commands::Run => {
            // FIX: Ako smo u system modu (imamo env var), isključi boje jer pišemo u file.
            // Ako smo u dev modu (terminal), ostavi boje.
            if is_system_mode_logging {
                tracing_subscriber::fmt().with_ansi(false).init();
            } else {
                tracing_subscriber::fmt().with_ansi(true).init();
            }

            let pid = process::id();
            if let Err(e) = fs::write(&pid_file_path, pid.to_string()) {
                error!("Failed to write PID file: {}", e);
            } else {
                info!("PID: {} written to {:?}", pid, pid_file_path);
            }
        }
    }

    // --- MAIN LOGIC ---

    info!("---------------------------------------------");
    info!("   VALTER ERP - DAEMON v0.1.0");
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

    let config_path = if is_system_mode && home_path.join("valter.config").exists() {
        home_path.join("valter.config")
    } else if PathBuf::from("valter.dev.config").exists() {
        PathBuf::from("valter.dev.config")
    } else if PathBuf::from("../valter.dev.config").exists() {
        PathBuf::from("../valter.dev.config")
    } else if PathBuf::from("valter.config").exists() {
        PathBuf::from("valter.config")
    } else {
        error!("CRITICAL: Configuration file not found.");
        let _ = fs::remove_file(&pid_file_path);
        return Err(anyhow::anyhow!("Config missing"));
    };

    let config = match Config::load(config_path.to_str().unwrap()) {
        Ok(cfg) => {
            info!("Loaded Configuration from: {:?}", config_path);
            Arc::new(cfg)
        }
        Err(e) => {
            error!("Config Parse Error: {}", e);
            let _ = fs::remove_file(&pid_file_path);
            return Err(e);
        }
    };

    let db_path = if is_system_mode {
        home_path.join("valter.db")
    } else if config_path.starts_with("..") {
        PathBuf::from("../valter.db")
    } else {
        PathBuf::from("valter.db")
    };

    let cloud = Arc::new(SqliteManager::new(db_path.to_str().unwrap())?);
    if let Err(e) = cloud.init_schema(&config) {
        error!("DB Schema Error: {}", e);
        let _ = fs::remove_file(&pid_file_path);
        return Err(e);
    }

    if let Ok(tools) = ToolGenerator::generate_tools(&config) {
        let _ = serde_json::to_string(&tools);
    }

    let cloud_clone = cloud.clone();
    let config_clone = config.clone();
    tokio::spawn(async move {
        if let Err(e) = api::start_server(cloud_clone, config_clone).await {
            error!("API Server failed: {}", e);
            process::exit(1);
        }
    });

    let processor = EventProcessor::new(cloud.clone(), config.clone());

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

    info!("VALTER Daemon is Running.");

    while let Some(event) = rx.recv().await {
        processor.handle_event(event).await;
    }

    let _ = fs::remove_file(&pid_file_path);
    Ok(())
}

fn is_daemon_running(pid_path: &Path) -> bool {
    if !pid_path.exists() {
        return false;
    }
    if let Ok(content) = fs::read_to_string(pid_path) {
        if let Ok(pid_int) = content.trim().parse::<i32>() {
            if signal::kill(Pid::from_raw(pid_int), None).is_ok() {
                return true;
            }
        }
    }
    let _ = fs::remove_file(pid_path);
    false
}

fn stop_daemon(pid_path: &Path) -> Result<()> {
    if !pid_path.exists() {
        println!("Valter is not running (PID file not found).");
        return Ok(());
    }

    let pid_str = fs::read_to_string(pid_path)?;
    let pid_int: i32 = pid_str.trim().parse()?;

    println!("Stopping Valter process (PID: {})...", pid_int);

    match signal::kill(Pid::from_raw(pid_int), Signal::SIGTERM) {
        Ok(_) => {
            println!("Signal sent. Waiting for process to exit...");
            for _ in 0..10 {
                if signal::kill(Pid::from_raw(pid_int), None).is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(500));
            }
            println!("Valter stopped.");
            let _ = fs::remove_file(pid_path);
        }
        Err(e) => {
            println!("Failed to stop process: {}", e);
            let _ = fs::remove_file(pid_path);
        }
    }
    Ok(())
}
