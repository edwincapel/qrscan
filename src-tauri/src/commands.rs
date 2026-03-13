use std::process::Command;

use serde::Serialize;

use crate::capture::{CaptureError, CaptureSession};
use crate::content_type::ParsedQRContent;

#[derive(Debug, Serialize, Clone)]
pub struct ScanResult {
    pub image_path: String,
    pub source_type: String,
}

/// Check if Screen Recording permission is granted.
#[tauri::command]
pub fn check_screen_permission() -> Result<bool, String> {
    let tmp = tempfile::Builder::new()
        .prefix("qrsnap_perm_")
        .suffix(".png")
        .tempfile()
        .map_err(|e| format!("Failed to create temp file: {e}"))?;

    let path = tmp.path().to_path_buf();

    let status = Command::new("screencapture")
        .arg("-x")
        .arg("-t")
        .arg("png")
        .arg(&path)
        .status()
        .map_err(|e| format!("Failed to run screencapture: {e}"))?;

    if !status.success() {
        return Ok(false);
    }

    let metadata = std::fs::metadata(&path)
        .map_err(|e| format!("Failed to read capture: {e}"))?;

    Ok(metadata.len() > 1024)
}

/// Trigger a scan. Mode is "region" or "window".
/// Returns the path to the captured PNG on success.
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
            // Persist the temp file so the frontend/worker can read it.
            // The NamedTempFile is consumed here — caller is responsible
            // for cleanup after decoding.
            let path = session.path().to_string_lossy().to_string();
            std::mem::forget(session);
            Ok(ScanResult {
                image_path: path,
                source_type: mode,
            })
        }
        Err(CaptureError::Cancelled) => Err("cancelled".to_string()),
        Err(CaptureError::PermissionDenied) => Err("permission_denied".to_string()),
        Err(CaptureError::Failed(msg)) => Err(msg),
    }
}

/// Parse raw QR content into structured data.
/// Rust is the canonical parser — frontend receives ParsedQRContent via IPC.
#[tauri::command]
pub fn parse_qr_content(raw: String) -> Result<ParsedQRContent, String> {
    Ok(crate::content_type::parse(&raw))
}
