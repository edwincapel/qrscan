/// Tiered scheme system — dedicated commands per scheme type.
/// See RFC-004 §10.2. No generic open passthrough.

/// Tier 1: Web URLs — highest risk. Only https/http allowed.
#[tauri::command]
pub fn open_url(url_str: String) -> Result<(), String> {
    let parsed = url::Url::parse(&url_str).map_err(|e| format!("Invalid URL: {e}"))?;
    match parsed.scheme() {
        "https" | "http" => opener::open(&url_str).map_err(|e| e.to_string()),
        scheme => Err(format!("Blocked scheme: {scheme}")),
    }
}

/// Tier 2: Email — validates address format, sanitizes subject.
#[tauri::command]
pub fn open_mailto(address: String, subject: Option<String>) -> Result<(), String> {
    if !is_valid_email(&address) {
        return Err("Invalid email address".into());
    }
    let clean_subject = subject.map(|s| {
        s.replace(['\r', '\n'], "").chars().take(256).collect::<String>()
    });
    let uri = match clean_subject {
        Some(s) => format!("mailto:{}?subject={}", address, urlencoding::encode(&s)),
        None => format!("mailto:{address}"),
    };
    opener::open(&uri).map_err(|e| e.to_string())
}

/// Tier 2: Phone — validates number format.
#[tauri::command]
pub fn open_tel(number: String) -> Result<(), String> {
    if !is_valid_phone(&number) {
        return Err("Invalid phone number".into());
    }
    let clean: String = number.chars().filter(|c| c.is_ascii_digit() || *c == '+').collect();
    opener::open(&format!("tel:{clean}")).map_err(|e| e.to_string())
}

/// Tier 2: SMS — validates number, caps body at 160 chars.
#[tauri::command]
pub fn open_sms(number: String, body: Option<String>) -> Result<(), String> {
    if !is_valid_phone(&number) {
        return Err("Invalid phone number".into());
    }
    let clean_num: String = number.chars().filter(|c| c.is_ascii_digit() || *c == '+').collect();
    let uri = match body.map(|b| b.chars().take(160).collect::<String>()) {
        Some(b) => format!("sms:{}?body={}", clean_num, urlencoding::encode(&b)),
        None => format!("sms:{clean_num}"),
    };
    opener::open(&uri).map_err(|e| e.to_string())
}

/// Tier 3: Geo — validates lat/lon, opens Apple Maps HTTPS URL.
#[tauri::command]
pub fn open_geo(lat: f64, lon: f64) -> Result<(), String> {
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
        return Err("Invalid coordinates".into());
    }
    let url = format!("https://maps.apple.com/?ll={lat},{lon}");
    opener::open(&url).map_err(|e| e.to_string())
}

/// Tier 4: Calendar — writes sanitized .ics to temp file, opens it.
#[tauri::command]
pub fn open_calendar_event(event_raw: String) -> Result<(), String> {
    let event = crate::ics_sanitizer::parse_vevent(&event_raw)
        .ok_or("Failed to parse calendar event")?;
    let ics_content = crate::ics_sanitizer::to_ics(&event);

    let tmp = tempfile::Builder::new()
        .prefix("qrsnap_event_")
        .suffix(".ics")
        .tempfile()
        .map_err(|e| format!("Temp file: {e}"))?;
    std::fs::write(tmp.path(), &ics_content)
        .map_err(|e| format!("Write .ics: {e}"))?;

    let path = tmp.path().to_path_buf();
    // Keep temp file alive until Calendar.app reads it
    std::mem::forget(tmp);
    opener::open(path).map_err(|e| e.to_string())
}

fn is_valid_email(s: &str) -> bool {
    let parts: Vec<&str> = s.split('@').collect();
    parts.len() == 2 && !parts[0].is_empty() && parts[1].contains('.') && !s.contains(' ')
}

fn is_valid_phone(s: &str) -> bool {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.len() >= 7 && digits.len() <= 15
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        assert!(is_valid_email("a@b.com"));
        assert!(!is_valid_email("no-at-sign"));
        assert!(!is_valid_email("has space@b.com"));
    }

    #[test]
    fn test_valid_phones() {
        assert!(is_valid_phone("+15551234567"));
        assert!(is_valid_phone("1234567"));
        assert!(!is_valid_phone("123"));
    }

    #[test]
    fn test_blocked_scheme() {
        let r = open_url("javascript:alert(1)".into());
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("Blocked scheme"));
    }

    #[test]
    fn test_blocked_data_scheme() {
        let r = open_url("data:text/html,<h1>hi</h1>".into());
        assert!(r.is_err());
    }

    #[test]
    fn test_invalid_email_rejected() {
        let r = open_mailto("not-an-email".into(), None);
        assert!(r.is_err());
    }

    #[test]
    fn test_invalid_phone_rejected() {
        let r = open_tel("123".into());
        assert!(r.is_err());
    }

    #[test]
    fn test_invalid_coords_rejected() {
        let r = open_geo(100.0, 0.0);
        assert!(r.is_err());
        let r = open_geo(0.0, 200.0);
        assert!(r.is_err());
    }
}
