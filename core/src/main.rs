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

    // ISPRAVAK: Definiraj produkcijski home direktorij JEDNOM i koristi ga za start/stop
    let default_prod_home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map(|h| PathBuf::from(h).join(".valter"))
        .unwrap_or_else(|_| PathBuf::from(".valter"));

    let command = cli.command.unwrap_or(Commands::Run);

    // Rano izađi za start/stop, koristeći fiksnu prod putanju
    if let Commands::Stop = command {
        let pid_file = default_prod_home.join("valter.pid");
        return stop_daemon(&pid_file);
    }
    if let Commands::Start = command {
        let pid_file = default_prod_home.join("valter.pid");
        if is_daemon_running(&pid_file) {
            println!("Stopping running instance...");
            let _ = stop_daemon(&pid_file);
            thread::sleep(Duration::from_millis(1000));
        }

        fs::create_dir_all(&default_prod_home)?;
        let log_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(default_prod_home.join("valter.log"))?;
        let exe = env::current_exe()?;

        process::Command::new(exe)
            .arg("run")
            .env("VALTER_HOME", &default_prod_home)
            .stdout(log_file.try_clone()?)
            .stderr(log_file)
            .spawn()?;

        println!("✅ Valter Started. Home: {:?}", default_prod_home);
        return Ok(());
    }

    // Odavde nadalje je samo 'run' komanda (bilo dev ili prod daemon)
    let env_home = env::var("VALTER_HOME").ok().map(PathBuf::from);
    let (valter_home, is_dev_mode) = if let Some(h) = env_home {
        (h, false) // Prod daemon
    } else {
        (env::current_dir()?, true) // Dev mode
    };

    let pid_file = valter_home.join("valter.pid");
    let timer = tracing_subscriber::fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string());
    tracing_subscriber::fmt().with_timer(timer).init();
    let _ = fs::write(&pid_file, process::id().to_string());
    info!(
        "VALTER ONLINE. Home: {:?}, DevMode: {}",
        valter_home, is_dev_mode
    );

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
                return Err(e.into());
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
        let processor_clone = processor.clone(); // Kloniraj za API
        let api_rx = shutdown_tx.subscribe();
        let api_handle = tokio::spawn(async move {
            // Ažurirani poziv s procesorom
            if let Err(e) =
                api::start_server(cloud_clone, config_clone, processor_clone, api_rx).await
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
    if let Ok(pid_str) = fs::read_to_string(pid_path) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            println!("Stopping PID {}...", pid);
            if signal::kill(Pid::from_raw(pid), Signal::SIGTERM).is_ok() {
                thread::sleep(Duration::from_millis(500));
                println!("Stopped.");
            } else {
                println!("Failed to send signal. Process might already be stopped.");
            }
        }
    }
    let _ = fs::remove_file(pid_path);
    Ok(())
}
