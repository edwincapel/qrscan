use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct ActionDefinition {
    pub id: String,
    pub label: String,
    pub payload: String,
    pub requires_confirmation: bool,
    pub confirmation_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParsedQRContent {
    pub content_type: String,
    pub raw: String,
    pub display_text: String,
    pub actions: Vec<ActionDefinition>,
    pub fields: Option<HashMap<String, String>>,
    pub warnings: Option<Vec<String>>,
}

const MAX_LEN: usize = 4096;

pub fn copy_action(text: &str) -> ActionDefinition {
    ActionDefinition {
        id: "copy_to_clipboard".into(),
        label: "Copy".into(),
        payload: text.to_string(),
        requires_confirmation: false,
        confirmation_message: None,
    }
}

pub fn confirm_action(id: &str, label: &str, payload: &str, msg: &str) -> ActionDefinition {
    ActionDefinition {
        id: id.into(),
        label: label.into(),
        payload: payload.into(),
        requires_confirmation: true,
        confirmation_message: Some(msg.into()),
    }
}

pub fn text_result(raw: &str) -> ParsedQRContent {
    ParsedQRContent {
        content_type: "text".into(),
        raw: raw.into(),
        display_text: raw.into(),
        actions: vec![copy_action(raw)],
        fields: None,
        warnings: None,
    }
}

pub fn parse(raw: &str) -> ParsedQRContent {
    if raw.len() > MAX_LEN {
        let truncated: String = raw.chars().take(MAX_LEN).collect();
        return ParsedQRContent {
            content_type: "error".into(),
            raw: truncated.clone(),
            display_text: "QR content too large to process".into(),
            actions: vec![copy_action(&truncated)],
            fields: None,
            warnings: None,
        };
    }
    if raw.starts_with("WIFI:") { return parse_wifi(raw); }
    if raw.starts_with("BEGIN:VCARD") { return parse_structured(raw, "vcard", "Contact Card"); }
    if raw.starts_with("BEGIN:VEVENT") { return parse_structured(raw, "event", "Calendar Event"); }
    if raw.starts_with("mailto:") { return parse_email_uri(raw); }
    if raw.starts_with("tel:") { return parse_phone(raw, raw.strip_prefix("tel:").unwrap_or(raw)); }
    if raw.starts_with("sms:") || raw.starts_with("smsto:") { return parse_sms(raw); }
    if raw.starts_with("geo:") { return parse_geo(raw); }
    if raw.starts_with("https://") || raw.starts_with("http://") { return parse_url(raw); }
    if raw.starts_with("www.") { return parse_url(&format!("https://{raw}")); }
    if is_email(raw) { return parse_email_uri(&format!("mailto:{raw}")); }
    if is_phone(raw) { return parse_phone(raw, raw); }
    text_result(raw)
}

fn parse_url(raw: &str) -> ParsedQRContent {
    let mut warnings = Vec::new();
    if raw.starts_with("http://") {
        warnings.push("This link is not encrypted (HTTP).".into());
    }
    let domain = url::Url::parse(raw)
        .ok()
        .and_then(|u| u.host_str().map(String::from))
        .unwrap_or_default();
    ParsedQRContent {
        content_type: "url".into(),
        raw: raw.into(),
        display_text: raw.into(),
        actions: vec![
            copy_action(raw),
            confirm_action("open_url", "Open URL", raw, &format!("Open {domain}?")),
        ],
        fields: Some([("domain".into(), domain)].into()),
        warnings: if warnings.is_empty() { None } else { Some(warnings) },
    }
}

fn parse_email_uri(raw: &str) -> ParsedQRContent {
    let addr = raw.strip_prefix("mailto:").unwrap_or(raw).split('?').next().unwrap_or("");
    ParsedQRContent {
        content_type: "email".into(),
        raw: raw.into(),
        display_text: addr.into(),
        actions: vec![
            copy_action(addr),
            confirm_action("open_mailto", "Compose Email", addr, &format!("Email {addr}?")),
        ],
        fields: Some([("address".into(), addr.into())].into()),
        warnings: None,
    }
}

fn parse_phone(raw: &str, number: &str) -> ParsedQRContent {
    ParsedQRContent {
        content_type: "phone".into(),
        raw: raw.into(),
        display_text: number.into(),
        actions: vec![
            copy_action(number),
            confirm_action("open_tel", "Call", number, &format!("Call {number}?")),
        ],
        fields: Some([("number".into(), number.into())].into()),
        warnings: None,
    }
}

fn parse_sms(raw: &str) -> ParsedQRContent {
    let rest = raw.strip_prefix("sms:").or_else(|| raw.strip_prefix("smsto:")).unwrap_or(raw);
    let number = rest.split('?').next().unwrap_or(rest);
    ParsedQRContent {
        content_type: "sms".into(),
        raw: raw.into(),
        display_text: number.into(),
        actions: vec![
            copy_action(number),
            confirm_action("open_sms", "Send Message", number, &format!("Message {number}?")),
        ],
        fields: Some([("number".into(), number.into())].into()),
        warnings: None,
    }
}

fn parse_geo(raw: &str) -> ParsedQRContent {
    let coords = raw.strip_prefix("geo:").unwrap_or("").split('?').next().unwrap_or("");
    let parts: Vec<&str> = coords.split(',').collect();
    let (lat, lon) = if parts.len() >= 2 { (parts[0], parts[1]) } else { (coords, "") };
    ParsedQRContent {
        content_type: "geo".into(),
        raw: raw.into(),
        display_text: format!("{lat}, {lon}"),
        actions: vec![
            copy_action(&format!("{lat}, {lon}")),
            confirm_action("open_geo", "Open in Maps", &format!("{lat},{lon}"), &format!("Open {lat}, {lon} in Maps?")),
        ],
        fields: Some([("lat".into(), lat.into()), ("lon".into(), lon.into())].into()),
        warnings: None,
    }
}

fn parse_wifi(raw: &str) -> ParsedQRContent {
    let body = raw.strip_prefix("WIFI:").unwrap_or("");
    let fields = parse_wifi_fields(body);
    let ssid = fields.get("S").cloned().unwrap_or_default();
    let password = fields.get("P").cloned().unwrap_or_default();
    let auth = fields.get("T").cloned().unwrap_or_else(|| "nopass".into());
    let hidden = fields.get("H").cloned().unwrap_or_else(|| "false".into());

    // Validate per RFC-004 §9.3
    if ssid.len() > 32 || password.len() > 63 {
        return text_result(raw);
    }
    let valid_auth = ["WPA", "WPA2", "WPA3", "WEP", "nopass"];
    if !valid_auth.contains(&auth.as_str()) {
        return text_result(raw);
    }

    let mut actions = vec![
        ActionDefinition {
            id: "copy_to_clipboard".into(),
            label: "Copy SSID".into(),
            payload: ssid.clone(),
            requires_confirmation: false,
            confirmation_message: None,
        },
    ];
    if !password.is_empty() {
        actions.push(ActionDefinition {
            id: "copy_to_clipboard".into(),
            label: "Copy Password".into(),
            payload: password.clone(),
            requires_confirmation: false,
            confirmation_message: None,
        });
    }

    let mut f = HashMap::new();
    f.insert("ssid".into(), ssid.clone());
    f.insert("auth".into(), auth);
    f.insert("hidden".into(), hidden);
    // Password NOT stored in fields — only available via action payload

    ParsedQRContent {
        content_type: "wifi".into(),
        raw: raw.into(),
        display_text: format!("{ssid} (WiFi)"),
        actions,
        fields: Some(f),
        warnings: None,
    }
}

/// Parse WiFi QR fields with escape handling: \; \, \\ \:
fn parse_wifi_fields(body: &str) -> HashMap<String, String> {
    let mut fields = HashMap::new();
    let mut chars = body.chars().peekable();
    while chars.peek().is_some() {
        // Read key (single char before ':')
        let key: String = (&mut chars).take_while(|&c| c != ':').collect();
        if key.is_empty() { break; }
        // Read value with escape handling, terminated by unescaped ';'
        let mut val = String::new();
        let mut prev_backslash = false;
        for c in chars.by_ref() {
            if prev_backslash {
                val.push(c); // \; \, \\ \: → literal char
                prev_backslash = false;
            } else if c == '\\' {
                prev_backslash = true;
            } else if c == ';' {
                break;
            } else {
                val.push(c);
            }
        }
        fields.insert(key, val);
    }
    fields
}

fn parse_structured(raw: &str, ctype: &str, label: &str) -> ParsedQRContent {
    ParsedQRContent {
        content_type: ctype.into(),
        raw: raw.into(),
        display_text: label.into(),
        actions: vec![copy_action(raw)],
        fields: None,
        warnings: None,
    }
}

fn is_email(s: &str) -> bool {
    let p: Vec<&str> = s.split('@').collect();
    p.len() == 2 && !p[0].is_empty() && p[1].contains('.') && !p[1].starts_with('.') && !s.contains(' ')
}

fn is_phone(s: &str) -> bool {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.len() >= 7 && digits.len() <= 15 && s.chars().all(|c| c.is_ascii_digit() || "+-() ".contains(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_https() {
        let r = parse("https://example.com/path");
        assert_eq!(r.content_type, "url");
        assert_eq!(r.actions.len(), 2);
        assert!(r.warnings.is_none());
    }

    #[test]
    fn test_url_http_warns() {
        let r = parse("http://example.com");
        assert_eq!(r.content_type, "url");
        assert!(r.warnings.is_some());
    }

    #[test]
    fn test_www_becomes_url() {
        let r = parse("www.example.com");
        assert_eq!(r.content_type, "url");
    }

    #[test]
    fn test_mailto() {
        let r = parse("mailto:test@example.com");
        assert_eq!(r.content_type, "email");
    }

    #[test]
    fn test_bare_email() {
        let r = parse("user@domain.com");
        assert_eq!(r.content_type, "email");
    }

    #[test]
    fn test_tel() {
        let r = parse("tel:+15551234567");
        assert_eq!(r.content_type, "phone");
    }

    #[test]
    fn test_bare_phone() {
        let r = parse("+1 (555) 123-4567");
        assert_eq!(r.content_type, "phone");
    }

    #[test]
    fn test_sms() {
        let r = parse("sms:+15551234567?body=hello");
        assert_eq!(r.content_type, "sms");
    }

    #[test]
    fn test_geo() {
        let r = parse("geo:40.7128,-74.0060");
        assert_eq!(r.content_type, "geo");
        let fields = r.fields.unwrap();
        assert_eq!(fields["lat"], "40.7128");
    }

    #[test]
    fn test_plain_text() {
        let r = parse("just some text");
        assert_eq!(r.content_type, "text");
    }

    #[test]
    fn test_too_long() {
        let long = "x".repeat(5000);
        let r = parse(&long);
        assert_eq!(r.content_type, "error");
    }

    #[test]
    fn test_wifi_basic() {
        let r = parse("WIFI:T:WPA;S:MyNetwork;P:secret123;;");
        assert_eq!(r.content_type, "wifi");
        let f = r.fields.unwrap();
        assert_eq!(f["ssid"], "MyNetwork");
        assert_eq!(f["auth"], "WPA");
    }

    #[test]
    fn test_wifi_escaped_semicolon() {
        let r = parse("WIFI:T:WPA;S:My\\;Net;P:pa\\;ss;;");
        assert_eq!(r.content_type, "wifi");
        let f = r.fields.unwrap();
        assert_eq!(f["ssid"], "My;Net");
        // Password in action payload, not fields
        assert_eq!(r.actions[1].payload, "pa;ss");
    }

    #[test]
    fn test_wifi_invalid_auth_falls_back() {
        let r = parse("WIFI:T:INVALID;S:Net;P:pass;;");
        assert_eq!(r.content_type, "text");
    }

    #[test]
    fn test_wifi_ssid_too_long() {
        let long_ssid = "A".repeat(33);
        let r = parse(&format!("WIFI:T:WPA;S:{long_ssid};P:pass;;"));
        assert_eq!(r.content_type, "text");
    }
}
