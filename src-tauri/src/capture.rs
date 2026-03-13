use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub enum CaptureError {
    Cancelled,
    PermissionDenied,
    Failed(String),
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => write!(f, "Capture cancelled"),
            Self::PermissionDenied => write!(f, "Screen Recording permission denied"),
            Self::Failed(msg) => write!(f, "Capture failed: {msg}"),
        }
    }
}

/// Holds a temp file that is deleted on drop, even on panic.
pub struct CaptureSession {
    path: PathBuf,
    _tempfile: tempfile::NamedTempFile,
}

impl CaptureSession {
    /// Create a new session with a secure temp file (0600 permissions).
    pub fn new() -> Result<Self, CaptureError> {
        let tmp = tempfile::Builder::new()
            .prefix("qrsnap_")
            .suffix(".png")
            .tempfile()
            .map_err(|e| CaptureError::Failed(format!("Temp file: {e}")))?;
        let path = tmp.path().to_path_buf();
        Ok(Self { path, _tempfile: tmp })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Run screencapture for interactive region selection.
    pub fn capture_region(&self) -> Result<(), CaptureError> {
        self.run_screencapture(&["-i"])
    }

    /// Run screencapture for interactive window selection.
    pub fn capture_window(&self) -> Result<(), CaptureError> {
        self.run_screencapture(&["-iw"])
    }

    fn run_screencapture(&self, args: &[&str]) -> Result<(), CaptureError> {
        let mut cmd = Command::new("screencapture");
        for arg in args {
            cmd.arg(arg);
        }
        cmd.arg("-t").arg("png").arg(&self.path);

        let status = cmd
            .status()
            .map_err(|e| CaptureError::Failed(format!("screencapture: {e}")))?;

        if !status.success() {
            return Err(CaptureError::Cancelled);
        }

        // 0-byte file means user pressed Esc
        let meta = std::fs::metadata(&self.path)
            .map_err(|e| CaptureError::Failed(format!("Read metadata: {e}")))?;
        if meta.len() == 0 {
            return Err(CaptureError::Cancelled);
        }

        Ok(())
    }
}
