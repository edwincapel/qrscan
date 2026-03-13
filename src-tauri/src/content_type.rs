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
    // Stub — full escape-aware parsing in PR 4.2
    text_result(raw)
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

