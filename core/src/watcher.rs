use anyhow::Result;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::path::Path;
use tokio::sync::mpsc;
use tracing::{error, info};

pub struct Watcher {
    // We keep the watcher alive here
    #[allow(dead_code)]
    watcher: RecommendedWatcher,
}

impl Watcher {
    pub fn new(path: &str, tx: mpsc::Sender<Event>) -> Result<Self> {
        let (std_tx, std_rx) = std::sync::mpsc::channel();

        let mut watcher = RecommendedWatcher::new(std_tx, Config::default())?;

        // Ensure the path exists before watching, or handle error gracefully
        if !Path::new(path).exists() {
            // For now, let's just warn and maybe create it or fail?
            // The plan says "Create a temp directory, modify a file..." so we should probably ensure it exists.
            // But for the general case, we should fail if the root path doesn't exist.
            // However, for testing I might want to create it.
            // Let's assume it exists for now as per "Green field (Empty Folder)" might mean project root.
            // I'll watch "." if path is invalid for now, or just fail.
        }

        watcher.watch(Path::new(path), RecursiveMode::Recursive)?;

        // Spawn a task to bridge std::mpsc to tokio::mpsc
        tokio::spawn(async move {
            // This is a blocking loop on the std channel, so we should run it in a blocking task or just spawn a regular thread.
            // But since we are in tokio, let's use spawn_blocking or just a dedicated thread.
            // Actually, std_rx.recv() blocks. Doing this in a simple tokio::spawn might block the executor thread if not careful,
            // but for a dedicated loop it's better to use spawn_blocking or a separate std::thread.
            // Let's use a separate std::thread for the bridge to be safe.
            std::thread::spawn(move || {
                while let Ok(res) = std_rx.recv() {
                    match res {
                        Ok(event) => {
                            // We need to send this to the async channel.
                            // blocking_send is available on mpsc::Sender if we weren't in async context,
                            // but here we are bridging to async.
                            // We can use handle.block_on or just use blocking_send if the channel is large enough.
                            // But tx is a tokio Sender.
                            // Simplest way: use blocking_send
                            if let Err(e) = tx.blocking_send(event) {
                                error!("Error sending event: {:?}", e);
                                break;
                            }
                        }
                        Err(e) => error!("Watch error: {:?}", e),
                    }
                }
            });
        });

        info!("Watcher initialized for path: {}", path);

        Ok(Self { watcher })
    }
}
