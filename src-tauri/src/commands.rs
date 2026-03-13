use std::process::Command;

/// Check if Screen Recording permission is granted by running a silent
/// screencapture and checking if the output file has non-zero size.
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

    // A blank/denied capture produces a very small file or 0 bytes
    // A real screen capture is typically >50KB
    Ok(metadata.len() > 1024)
}
