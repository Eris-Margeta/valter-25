use anyhow::Result;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::path::Path;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

pub struct Watcher {
    #[allow(dead_code)]
    watcher: RecommendedWatcher,
}

impl Watcher {
    // CHANGE: Accepts a list of paths
    pub fn new(paths: Vec<String>, tx: mpsc::Sender<Event>) -> Result<Self> {
        let (std_tx, std_rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(std_tx, Config::default())?;

        for path_str in paths {
            let path = Path::new(&path_str);
            if path.exists() {
                info!("👁️  Watcher attached to: {:?}", path);
                if let Err(e) = watcher.watch(path, RecursiveMode::Recursive) {
                    error!("Failed to watch {:?}: {}", path, e);
                }
            } else {
                warn!("⚠️  Cannot watch non-existent path: {:?}", path);
            }
        }

        // Bridge thread (std channel -> tokio channel)
        tokio::spawn(async move {
            // Using spawn_blocking for the blocking receive loop would be ideal,
            // but a separate thread via std::thread inside main loop is also fine.
            // Here we just use a dedicated thread to not block tokio worker.
            std::thread::spawn(move || {
                while let Ok(res) = std_rx.recv() {
                    match res {
                        Ok(event) => {
                            if let Err(e) = tx.blocking_send(event) {
                                error!("Error sending event to async channel: {:?}", e);
                                break;
                            }
                        }
                        Err(e) => error!("Watch error: {:?}", e),
                    }
                }
            });
        });

        Ok(Self { watcher })
    }
}
