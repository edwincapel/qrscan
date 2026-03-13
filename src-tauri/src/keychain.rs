/// macOS Keychain access via `security` CLI.
/// ALL args passed via .arg() — NO shell string interpolation.
/// See RFC-004 §10.3.

use std::process::Command;

const SERVICE: &str = "QRSnap";

#[allow(dead_code)]
pub fn store(ssid: &str, password: &str) -> Result<(), String> {
    if ssid.len() > 32 {
        return Err("SSID too long (max 32 bytes)".into());
    }
    if password.len() > 63 {
        return Err("Password too long (max 63 chars)".into());
    }

    let status = Command::new("security")
        .arg("add-generic-password")
        .arg("-s").arg(SERVICE)
        .arg("-a").arg(ssid)
        .arg("-w").arg(password)
        .arg("-U") // Update if exists
        .status()
        .map_err(|e| format!("Keychain write: {e}"))?;

    if !status.success() {
        return Err("Keychain write failed".into());
    }
    Ok(())
}

pub fn retrieve(ssid: &str) -> Result<Option<String>, String> {
    let output = Command::new("security")
        .arg("find-generic-password")
        .arg("-s").arg(SERVICE)
        .arg("-a").arg(ssid)
        .arg("-w")
        .output()
        .map_err(|e| format!("Keychain read: {e}"))?;

    if !output.status.success() {
        return Ok(None); // Not found is not an error
    }

    String::from_utf8(output.stdout)
        .map(|s| Some(s.trim().to_string()))
        .map_err(|_| "Keychain: invalid UTF-8".into())
}

pub fn delete(ssid: &str) -> Result<(), String> {
    let _ = Command::new("security")
        .arg("delete-generic-password")
        .arg("-s").arg(SERVICE)
        .arg("-a").arg(ssid)
        .status(); // Ignore errors — entry may not exist
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssid_too_long_rejected() {
        let long = "A".repeat(33);
        assert!(store(&long, "pass").is_err());
    }

    #[test]
    fn test_password_too_long_rejected() {
        let long = "A".repeat(64);
        assert!(store("net", &long).is_err());
    }
}
