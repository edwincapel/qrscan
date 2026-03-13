use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const MAX_ENTRIES: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanEntry {
    pub id: String,
    pub scanned_at: String,
    pub result: String,
    pub result_type: String,
    pub parsed_data: Option<std::collections::HashMap<String, String>>,
    pub source_type: String,
    pub source_name: Option<String>,
    pub thumbnail_file: Option<String>,
}

fn history_dir() -> Result<PathBuf, String> {
    let dir = dirs::data_dir()
        .ok_or("No data directory")?
        .join("com.qrsnap.app");
    fs::create_dir_all(&dir).map_err(|e| format!("Create dir: {e}"))?;
    Ok(dir)
}

fn history_path() -> Result<PathBuf, String> {
    Ok(history_dir()?.join("history.json"))
}

fn thumbnails_dir() -> Result<PathBuf, String> {
    let dir = history_dir()?.join("thumbnails");
    fs::create_dir_all(&dir).map_err(|e| format!("Create thumbnails dir: {e}"))?;
    Ok(dir)
}

pub fn load_entries() -> Result<Vec<ScanEntry>, String> {
    let path = history_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(&path).map_err(|e| format!("Read history: {e}"))?;
    serde_json::from_str(&data).map_err(|e| format!("Parse history: {e}"))
}

fn save_entries(entries: &[ScanEntry]) -> Result<(), String> {
    let path = history_path()?;
    let data = serde_json::to_string_pretty(entries).map_err(|e| format!("Serialize: {e}"))?;
    fs::write(&path, data).map_err(|e| format!("Write history: {e}"))
}

pub fn add_entry(entry: ScanEntry) -> Result<(), String> {
    let mut entries = load_entries()?;
    entries.insert(0, entry);
    // FIFO purge — delete oldest entries beyond MAX_ENTRIES
    while entries.len() > MAX_ENTRIES {
        if let Some(old) = entries.pop() {
            delete_thumbnail(&old.thumbnail_file);
        }
    }
    save_entries(&entries)
}

pub fn delete_entry(id: &str) -> Result<Option<ScanEntry>, String> {
    let mut entries = load_entries()?;
    let idx = entries.iter().position(|e| e.id == id);
    if let Some(i) = idx {
        let removed = entries.remove(i);
        delete_thumbnail(&removed.thumbnail_file);
        save_entries(&entries)?;
        Ok(Some(removed))
    } else {
        Ok(None)
    }
}

pub fn clear_all() -> Result<Vec<ScanEntry>, String> {
    let entries = load_entries()?;
    for e in &entries {
        delete_thumbnail(&e.thumbnail_file);
    }
    save_entries(&[])?;
    Ok(entries)
}

fn delete_thumbnail(file: &Option<String>) {
    if let Some(name) = file {
        if let Ok(dir) = thumbnails_dir() {
            let _ = fs::remove_file(dir.join(name));
        }
    }
}

#[allow(dead_code)]
pub fn save_thumbnail(id: &str, data: &[u8]) -> Result<String, String> {
    let dir = thumbnails_dir()?;
    let filename = format!("{id}.png");
    fs::write(dir.join(&filename), data).map_err(|e| format!("Write thumbnail: {e}"))?;
    Ok(filename)
}
