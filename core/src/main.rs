mod config;
mod watcher;
mod context_engine;
mod cloud;
mod api;
mod oracle;
mod processor;
mod aggregator;
mod fs_writer;

use clap::{Parser, Subcommand};
use config::Config;
use watcher::Watcher;
use cloud::SqliteManager;
use oracle::ToolGenerator;
use processor::EventProcessor;
use anyhow::Result;
use tracing::{info, error};
use tokio::sync::mpsc;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use std::process;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

// CLI Definition
#[derive(Parser)]
#[command(name = "valter")]
#[command(about = "Valter ERP Daemon", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the daemon in the background (Production Mode)
    Start,
    /// Stop the running daemon
    Stop,
    /// Run in foreground
    Run,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Učitaj env vars ako postoje
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    // 1. Helper za detekciju System Home-a (~/.valter)
    let user_home = env::var("HOME").or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let default_system_home = PathBuf::from(user_home).join(".valter");

    // 2. Trenutni VALTER_HOME (ako je postavljen)
    let env_valter_home = env::var("VALTER_HOME").ok().map(PathBuf::from);

    // Određujemo stvarni home za trenutni proces
    let current_home = env_valter_home.clone().unwrap_or_else(|| PathBuf::from("."));
    
    // PID file lokacija
    let pid_file_path = if env_valter_home.is_some() {
        // Ako je forsiran system mode, pid je tamo
        env_valter_home.unwrap().join("valter.pid")
    } else if current_home.join("valter.dev.config").exists() {
        // Dev mode root
        PathBuf::from("valter.pid")
    } else {
        // Default system location
        default_system_home.join("valter.pid")
    };

    match cli.command.unwrap_or(Commands::Run) {
        Commands::Stop => {
            return stop_daemon(&pid_file_path);
        }
        Commands::Start => {
            println!("🚀 Initializing Valter Daemon...");
            
            // FORSIRAMO PRODUKCIJU:
            // Ako korisnik nije eksplicitno postavio VALTER_HOME,
            // mi ga postavljamo na ~/.valter za child proces.
            let target_home = env_valter_home.unwrap_or(default_system_home);
            
            // Osiguraj da folder postoji
            fs::create_dir_all(&target_home)?;

            // Log file setup (Append mode)
            let log_path = target_home.join("valter.log");
            let log_file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_path)?;

            let exe = env::current_exe()?;
            
            println!("   Home: {:?}", target_home);
            println!("   Logs: {:?}", log_path);

            process::Command::new(exe)
                .arg("run") // Child runs in 'run' mode
                .env("VALTER_HOME", &target_home) // FORCE ENV VAR
                .stdout(log_file.try_clone()?) // Redirect stdout to file
                .stderr(log_file) // Redirect stderr to file
                .spawn()?; // Detached process
            
            println!("✅ Valter started in background.");
            return Ok(());
        }
        Commands::Run => {
            // Setup logging for the runner process
            tracing_subscriber::fmt::init();
            
            // Write PID file
            let pid = process::id();
            if let Err(e) = fs::write(&pid_file_path, pid.to_string()) {
                error!("Failed to write PID file to {:?}: {}", pid_file_path, e);
            } else {
                info!("PID: {} written to {:?}", pid, pid_file_path);
            }
            
            // Nastavi na main logic...
        }
    }

    // --- MAIN LOGIC (Foreground) ---
    // Ovdje dolazimo samo ako je komanda 'Run' (ili default)

    info!("---------------------------------------------");
    info!("   VALTER ERP - DAEMON v0.1.0");
    info!("   Open Source Edition (MIT)");
    info!("---------------------------------------------");

    // Ponovno evaluiramo mode jer nas je možda Start komanda pozvala s novim ENV
    let valter_home = env::var("VALTER_HOME").unwrap_or_else(|_| ".".to_string());
    let home_path = PathBuf::from(&valter_home);
    let is_system_mode = valter_home != ".";

    if is_system_mode {
        info!("Mode: SYSTEM (Home: {:?})", home_path);
    } else {
        info!("Mode: DEVELOPMENT / LOCAL");
    }

    // Config Loading Strategy
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
        return Err(anyhow::anyhow!("Config missing"));
    };

    let config = match Config::load(config_path.to_str().unwrap()) {
        Ok(cfg) => {
            info!("Loaded Configuration from: {:?}", config_path);
            Arc::new(cfg)
        },
        Err(e) => {
            error!("Config Parse Error: {}", e);
            let _ = fs::remove_file(&pid_file_path);
            return Err(e);
        }
    };

    // Database Setup
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

    // Oracle Generation
    if let Ok(tools) = ToolGenerator::generate_tools(&config) {
        let _ = serde_json::to_string(&tools);
    }

    // API Server
    let cloud_clone = cloud.clone();
    let config_clone = config.clone();
    tokio::spawn(async move {
        if let Err(e) = api::start_server(cloud_clone, config_clone).await {
            error!("API Server failed: {}", e);
            process::exit(1);
        }
    });

    // Processor & Watcher
    let processor = EventProcessor::new(cloud.clone(), config.clone());
    
    let watch_root = if is_system_mode {
        // U produkciji ne želimo nužno gledati home folder osim ako config ne kaže drugačije.
        // Ali za sada, neka gleda home jer tamo može biti config.
        // Još bolje: Gledajmo ono što piše u configu.
        // Ali Watcher mora imati root path.
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

fn stop_daemon(pid_path: &Path) -> Result<()> {
    if !pid_path.exists() {
        println!("Valter is not running (PID file not found at {:?}).", pid_path);
        return Ok(());
    }

    let pid_str = fs::read_to_string(pid_path)?;
    let pid_int: i32 = pid_str.trim().parse()?;
    
    println!("Stopping Valter process (PID: {})...", pid_int);
    
    match signal::kill(Pid::from_raw(pid_int), Signal::SIGTERM) {
        Ok(_) => {
            println!("Valter stopped successfully.");
            let _ = fs::remove_file(pid_path);
        }
        Err(e) => {
            println!("Failed to stop process: {}", e);
            let _ = fs::remove_file(pid_path);
        }
    }
    Ok(())
}

