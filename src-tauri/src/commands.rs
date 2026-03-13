use std::process::Command;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::capture::{CaptureError, CaptureSession};
use crate::content_type::ParsedQRContent;

/// Show the main panel window from Rust — more reliable than JS window APIs.
#[tauri::command]
pub fn show_panel_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        let _ = window.set_always_on_top(true);
        let _ = window.set_focus();
    }
    Ok(())
}

/// Hide the main panel window.
#[tauri::command]
pub fn hide_panel_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_always_on_top(false);
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(Debug, Serialize, Clone)]
pub struct ScanResult {
    /// Base64-encoded PNG. Read in Rust before Drop deletes the temp file.
    pub image_data: String,
    pub source_type: String,
}

/// Check if Screen Recording permission is granted.
/// Uses the same byte-diversity heuristic as validate_capture() in capture.rs.
#[tauri::command]
pub fn check_screen_permission() -> Result<bool, String> {
    let tmp = tempfile::Builder::new()
        .prefix("qrsnap_perm_")
        .suffix(".png")
        .tempfile()
        .map_err(|e| format!("Temp file: {e}"))?;

    let status = Command::new("screencapture")
        .arg("-x").arg("-t").arg("png").arg(tmp.path())
        .status()
        .map_err(|e| format!("screencapture: {e}"))?;

    if !status.success() {
        return Ok(false);
    }

    let meta = std::fs::metadata(tmp.path())
        .map_err(|e| format!("Read metadata: {e}"))?;

    if meta.len() == 0 {
        return Ok(false);
    }

    // Same byte-diversity check as validate_capture()
    // A blank/denied PNG uses very few unique byte values (<60).
    // A real screenshot uses many (>100).
    if meta.len() < 5000 {
        let data = std::fs::read(tmp.path())
            .map_err(|e| format!("Read file: {e}"))?;
        let mut seen = [false; 256];
        for &b in &data { seen[b as usize] = true; }
        let unique = seen.iter().filter(|&&v| v).count();
        return Ok(unique >= 60);
    }

    Ok(true)
}

/// Trigger a scan. Mode is "region" or "window".
/// Reads image bytes into memory before session drops (deleting temp file).
/// Returns base64-encoded PNG — no file path exposed to frontend.
#[tauri::command]
pub fn trigger_scan(mode: String) -> Result<ScanResult, String> {
    let session = CaptureSession::new().map_err(|e| e.to_string())?;

    let result = match mode.as_str() {
        "region" => session.capture_region(),
        "window" => session.capture_window(),
        other => return Err(format!("Invalid scan mode: {other}")),
    };

    match result {
        Ok(()) => {
            let bytes = std::fs::read(session.path())
                .map_err(|e| format!("Read capture: {e}"))?;
            // session drops here — temp file deleted immediately
            Ok(ScanResult {
                image_data: encode_base64(&bytes),
                source_type: mode,
            })
        }
        Err(CaptureError::Cancelled) => Err("cancelled".to_string()),
        Err(CaptureError::PermissionDenied) => Err("permission_denied".to_string()),
        Err(CaptureError::Failed(msg)) => Err(msg),
    }
}

/// Minimal base64 encoder — avoids external dependency for small image payloads.
fn encode_base64(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[(n >> 18) & 63] as char);
        out.push(CHARS[(n >> 12) & 63] as char);
        out.push(if chunk.len() > 1 { CHARS[(n >> 6) & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { CHARS[n & 63] as char } else { '=' });
    }
    out
}

/// Parse raw QR content into structured data.
#[tauri::command]
pub fn parse_qr_content(raw: String) -> Result<ParsedQRContent, String> {
    Ok(crate::content_type::parse(&raw))
}

#[tauri::command]
pub fn get_history() -> Result<Vec<crate::history::ScanEntry>, String> {
    crate::history::load_entries()
}

#[tauri::command]
pub fn save_scan(entry: crate::history::ScanEntry) -> Result<(), String> {
    crate::history::add_entry(entry)
}

/// Delete a history entry. Cascades: removes thumbnail + Keychain if WiFi.
#[tauri::command]
pub fn delete_history(id: String) -> Result<(), String> {
    let removed = crate::history::delete_entry(&id)?;
    if let Some(entry) = removed {
        if entry.result_type == "wifi" {
            if let Some(fields) = &entry.parsed_data {
                if let Some(ssid) = fields.get("ssid") {
                    let _ = crate::keychain::delete(ssid);
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn clear_history() -> Result<(), String> {
    let entries = crate::history::clear_all()?;
    for entry in &entries {
        if entry.result_type == "wifi" {
            if let Some(fields) = &entry.parsed_data {
                if let Some(ssid) = fields.get("ssid") {
                    let _ = crate::keychain::delete(ssid);
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn keychain_get_password(ssid: String) -> Result<Option<String>, String> {
    crate::keychain::retrieve(&ssid)
}

#[tauri::command]
pub fn get_settings() -> Result<String, String> {
    let path = settings_path()?;
    if !path.exists() { return Ok("{}".into()); }
    std::fs::read_to_string(&path).map_err(|e| format!("Read settings: {e}"))
}

#[tauri::command]
pub fn save_settings(json: String) -> Result<(), String> {
    let path = settings_path()?;
    std::fs::write(&path, json).map_err(|e| format!("Write settings: {e}"))
}

fn settings_path() -> Result<std::path::PathBuf, String> {
    let dir = dirs::data_dir().ok_or("No data dir")?.join("com.qrsnap.app");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Create dir: {e}"))?;
    Ok(dir.join("settings.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_hello() {
        assert_eq!(encode_base64(b"Hello"), "SGVsbG8=");
    }

    #[test]
    fn test_base64_roundtrip_padding() {
        assert_eq!(encode_base64(b"Man"), "TWFu");
        assert_eq!(encode_base64(b"Ma"), "TWE=");
        assert_eq!(encode_base64(b"M"), "TQ==");
    }
}
