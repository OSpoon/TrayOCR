use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: u64,
    pub text: String,
    pub timestamp: String,
}

fn history_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("history.json")
}

pub fn load(app_data_dir: &Path) -> Vec<HistoryEntry> {
    let path = history_path(app_data_dir);
    if !path.exists() {
        return Vec::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn append(app_data_dir: &Path, entry: HistoryEntry) {
    let mut entries = load(app_data_dir);
    entries.insert(0, entry);
    if entries.len() > 200 {
        entries.truncate(200);
    }
    let _ = std::fs::create_dir_all(app_data_dir);
    if let Ok(content) = serde_json::to_string_pretty(&entries) {
        let _ = std::fs::write(history_path(app_data_dir), content);
    }
}

pub fn clear(app_data_dir: &Path) {
    let path = history_path(app_data_dir);
    let _ = std::fs::remove_file(path);
}
