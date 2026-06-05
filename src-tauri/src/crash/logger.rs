use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A single crash log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrashEntry {
    pub id: String,
    pub timestamp: String,
    pub error_type: String,
    pub message: String,
    pub stack_trace: Option<String>,
    pub app_version: String,
    pub os_info: String,
    /// Sanitized context (no file paths, usernames, etc.)
    #[serde(default)]
    pub context: serde_json::Value,
}

pub struct CrashLogger {
    log_dir: PathBuf,
    max_entries: usize,
    /// Mutex to serialize file I/O operations across threads
    io_lock: std::sync::Mutex<()>,
}

impl CrashLogger {
    pub fn new() -> Option<Self> {
        let log_dir = dirs::data_dir()?.join(crate::APP_DATA_DIR).join("crash_logs");
        fs::create_dir_all(&log_dir).ok()?;
        Some(Self {
            log_dir,
            max_entries: 50,
            io_lock: std::sync::Mutex::new(()),
        })
    }

    /// Log a crash event to a JSON file
    pub fn log_crash(&self, error: &crate::error::AppError) -> Option<CrashEntry> {
        let _guard = self.io_lock.lock().ok()?;
        let entry = CrashEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            error_type: error.error_type().to_string(),
            message: error.sanitized_message(),
            stack_trace: None,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            os_info: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
            context: serde_json::json!({}),
        };

        let file_path = self.log_dir.join(format!("crash_{}.json", &entry.id[..8]));
        let json = serde_json::to_string_pretty(&entry).ok()?;
        fs::write(&file_path, json).ok()?;

        log::info!("Crash logged: {} -> {}", entry.id, file_path.display());

        // Prune old entries if over limit
        self.prune_old_entries_locked();

        Some(entry)
    }

    /// Log a panic (call from std::panic::set_hook).
    /// Uses try_lock to avoid deadlock if panic occurred while io_lock was held.
    pub fn log_panic(&self, panic_info: &str) -> Option<CrashEntry> {
        // Use try_lock — if the mutex is poisoned or already locked (e.g., panic
        // happened during log_crash), skip logging to avoid deadlock.
        let _guard = match self.io_lock.try_lock() {
            Ok(g) => g,
            Err(_) => {
                eprintln!("[CrashLogger] Could not acquire lock in panic handler, skipping");
                return None;
            }
        };
        let entry = CrashEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            error_type: "panic".to_string(),
            message: sanitize_panic_message(panic_info),
            stack_trace: None,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            os_info: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
            context: serde_json::json!({}),
        };

        let file_path = self.log_dir.join(format!("panic_{}.json", &entry.id[..8]));
        let json = match serde_json::to_string_pretty(&entry) {
            Ok(j) => j,
            Err(_) => return None,
        };
        let _ = fs::write(&file_path, json);

        eprintln!("[CrashLogger] Panic logged: {}", file_path.display());
        Some(entry)
    }

    /// List all crash log entries
    pub fn list_entries(&self) -> Vec<CrashEntry> {
        let _guard = self.io_lock.lock().ok();
        let mut entries = Vec::new();
        if let Ok(dir) = fs::read_dir(&self.log_dir) {
            for entry in dir.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(crash) = serde_json::from_str::<CrashEntry>(&content) {
                            entries.push(crash);
                        }
                    }
                }
            }
        }
        // Sort by timestamp descending
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries
    }

    /// Delete a specific crash entry
    pub fn delete_entry(&self, id: &str) -> bool {
        let _guard = match self.io_lock.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };
        if let Ok(dir) = fs::read_dir(&self.log_dir) {
            for entry in dir.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    if name.ends_with(&id[..8.min(id.len())]) {
                        return fs::remove_file(&path).is_ok();
                    }
                }
            }
        }
        false
    }

    /// Delete all crash logs
    pub fn clear_all(&self) -> usize {
        let _guard = match self.io_lock.lock() {
            Ok(g) => g,
            Err(_) => return 0,
        };
        let mut count = 0;
        if let Ok(dir) = fs::read_dir(&self.log_dir) {
            for entry in dir.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    if fs::remove_file(&path).is_ok() {
                        count += 1;
                    }
                }
            }
        }
        log::info!("Cleared {} crash log entries", count);
        count
    }

    /// Generate a GitHub Issue URL pre-filled with crash info
    pub fn generate_issue_url(&self, entry: &CrashEntry, owner: &str, repo: &str) -> String {
        let title = url_encode(&format!(
            "[Crash] {} - {}",
            entry.error_type,
            truncate(&entry.message, 60)
        ));
        let body = url_encode(&format!(
            "## Crash Report\n\n\
            **App Version:** {}\n\
            **OS:** {}\n\
            **Error Type:** {}\n\
            **Timestamp:** {}\n\n\
            ### Error Message\n```\n{}\n```\n\n\
            ### Steps to Reproduce\n(please describe what you were doing)\n\n",
            entry.app_version,
            entry.os_info,
            entry.error_type,
            entry.timestamp,
            entry.message,
        ));
        format!(
            "https://github.com/{}/{}/issues/new?title={}&body={}",
            owner, repo, title, body
        )
    }

    /// Prune old entries (must be called while io_lock is already held)
    fn prune_old_entries_locked(&self) {
        let mut entries = Vec::new();
        if let Ok(dir) = fs::read_dir(&self.log_dir) {
            for entry in dir.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(crash) = serde_json::from_str::<CrashEntry>(&content) {
                            entries.push(crash);
                        }
                    }
                }
            }
        }
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        if entries.len() > self.max_entries {
            let to_remove = entries.len() - self.max_entries;
            for entry in entries.iter().rev().take(to_remove) {
                let prefix = &entry.id[..8.min(entry.id.len())];
                if let Ok(dir) = fs::read_dir(&self.log_dir) {
                    for de in dir.flatten() {
                        let p = de.path();
                        if let Some(name) = p.file_stem().and_then(|s| s.to_str()) {
                            if name.ends_with(prefix) {
                                let _ = fs::remove_file(&p);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Sanitize panic message: remove file paths and usernames
fn sanitize_panic_message(msg: &str) -> String {
    let mut sanitized = msg.to_string();
    // Remove common path patterns
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy().to_string();
        sanitized = sanitized.replace(&home_str, "~");
    }
    // Remove Windows user paths
    sanitized = regex_replace_paths(&sanitized);
    sanitized
}

fn regex_replace_paths(s: &str) -> String {
    let mut result = s.to_string();
    // Replace all C:\Users\xxx patterns (loop until no more matches)
    loop {
        if let Some(start) = result.find("C:\\Users\\") {
            if let Some(end) = result[start + 10..].find('\\').map(|i| start + 10 + i) {
                result = format!("{}~\\{}", &result[..start], &result[end..]);
                continue;
            }
        }
        break;
    }
    // Replace all /Users/xxx or /home/xxx patterns
    for prefix in &["/Users/", "/home/", "/usr/", "/tmp/", "/var/folders/"] {
        loop {
            if let Some(start) = result.find(prefix) {
                let after = start + prefix.len();
                if let Some(end) = result[after..].find('/').map(|i| after + i) {
                    result = format!("{}~{}", &result[..start], &result[end..]);
                    continue;
                }
            }
            break;
        }
    }
    // Replace Windows APPDATA/LOCALAPPDATA paths (e.g., C:\Users\xxx\AppData\...)
    loop {
        if let Some(start) = result.find("C:\\Users\\") {
            // Already handled above, but also match nested patterns
            if let Some(end) = result[start + 10..].find('\\').map(|i| start + 10 + i) {
                result = format!("{}~\\{}", &result[..start], &result[end..]);
                continue;
            }
        }
        break;
    }
    result
}

fn url_encode(s: &str) -> String {
    s.replace('%', "%25")
        .replace(' ', "%20")
        .replace('\n', "%0A")
        .replace('#', "%23")
        .replace('&', "%26")
        .replace('=', "%3D")
        .replace('+', "%2B")
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        // Find the nearest char boundary at or before max
        let end = s.char_indices()
            .take_while(|(i, _)| *i < max)
            .last()
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        format!("{}...", &s[..end])
    }
}
