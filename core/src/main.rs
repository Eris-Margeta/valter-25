use anyhow::Result;
use clap::{Parser, Subcommand};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::thread;
use std::time::Duration;

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
    if env::var("VALTER_TEST_MODE").is_err() {
        dotenv::dotenv().ok();
    }
    let cli = Cli::parse();

    let default_prod_home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map(|h| PathBuf::from(h).join(".valter"))
        .unwrap_or_else(|_| PathBuf::from(".valter"));

    let command = cli.command.unwrap_or(Commands::Run);

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

    // Initialize logging for RUN command
    let timer = tracing_subscriber::fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string());
    tracing_subscriber::fmt().with_timer(timer).init();

    let env_home = env::var("VALTER_HOME").ok().map(PathBuf::from);
    let (valter_home, is_dev_mode) = if let Some(h) = env_home {
        (h, false) // Prod daemon
    } else {
        (env::current_dir()?, true) // Dev mode
    };

    valter_core::run(valter_home, is_dev_mode).await
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
