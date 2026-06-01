use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::collectors::CollectorManager;
use crate::error::AppResult;

pub struct FileWatcher {
    watcher: Option<RecommendedWatcher>,
    receiver: Option<std::sync::mpsc::Receiver<notify::Result<Event>>>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            watcher: None,
            receiver: None,
        }
    }

    /// Start watching multiple paths
    pub fn watch_paths(&mut self, paths: &[PathBuf]) -> AppResult<()> {
        let (tx, rx) = std::sync::mpsc::channel();
        // Always store the receiver so start_event_loop won't panic,
        // even if no paths end up being watched.
        self.receiver = Some(rx);

        let mut watcher = notify::recommended_watcher(move |res| {
            tx.send(res).ok();
        })?;

        watcher.configure(Config::default())?;

        let mut watched_count = 0;
        for path in paths {
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)?;
                watched_count += 1;
                log::info!("Watching: {:?}", path);
            } else {
                log::info!("Path not found, skipping: {:?}", path);
            }
        }

        log::info!("File watcher active on {} paths", watched_count);
        self.watcher = Some(watcher);
        Ok(())
    }

    /// Consume the receiver and start the event loop in a background thread.
    /// This method takes ownership of the receiver (can only be called once).
    pub fn start_event_loop(
        &mut self,
        manager: Arc<CollectorManager>,
        app_handle: tauri::AppHandle,
    ) -> AppResult<()> {
        let rx = self
            .receiver
            .take()
            .ok_or_else(|| crate::error::AppError::Config("Event receiver already consumed".into()))?;

        std::thread::Builder::new()
            .name("tokenowl-watcher".into())
            .spawn(move || {
                Self::event_loop(rx, manager, app_handle);
            })
            .map_err(|e| crate::error::AppError::Io(e))?;

        Ok(())
    }

    /// Internal event loop with debouncing
    fn event_loop(
        rx: std::sync::mpsc::Receiver<notify::Result<Event>>,
        manager: Arc<CollectorManager>,
        app_handle: tauri::AppHandle,
    ) {
        log::info!("File watcher event loop started");

        // Debounce: track last processed time per file path
        let mut last_processed: HashMap<PathBuf, Instant> = HashMap::new();
        let debounce_duration = Duration::from_secs(2);

        for event_result in rx.iter() {
            match event_result {
                Ok(event) => {
                    // Only process create and modify events
                    let is_relevant = matches!(
                        event.kind,
                        EventKind::Create(_) | EventKind::Modify(_)
                    );

                    if !is_relevant {
                        continue;
                    }

                    for path in event.paths {
                        // Check if this file type matches any collector
                        if !manager.matches_any_collector(&path) {
                            continue;
                        }

                        // Debounce: skip if we processed this file recently
                        let now = Instant::now();
                        if let Some(last) = last_processed.get(&path) {
                            if now.duration_since(*last) < debounce_duration {
                                continue;
                            }
                        }
                        last_processed.insert(path.clone(), now);

                        // Process the file change
                        match manager.process_file_change(&path) {
                            Ok(true) => {
                                log::info!("Data updated from: {:?}", path);
                                // Emit event to frontend
                                use tauri::Emitter;
                                let _ = app_handle.emit("tokenowl:data-changed", ());
                            }
                            Ok(false) => {
                                // No new data (expected for some events)
                            }
                            Err(e) => {
                                log::warn!("Error processing {:?}: {}", path, e);
                            }
                        }
                    }

                    // Periodic cleanup of old debounce entries
                    if last_processed.len() > 500 {
                        let cutoff = Instant::now() - Duration::from_secs(60);
                        last_processed.retain(|_, v| *v > cutoff);
                    }
                }
                Err(e) => {
                    log::error!("Watcher error: {}", e);
                }
            }
        }

        log::info!("File watcher event loop exited");
    }
}
