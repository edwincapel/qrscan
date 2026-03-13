/// Sanitize a VEVENT QR payload into a safe .ics file.
/// Only allowlisted properties are included. Everything else is stripped.
/// See RFC-004 §9.4.

const MAX_SUMMARY: usize = 256;
const MAX_LOCATION: usize = 256;
const MAX_DESCRIPTION: usize = 1024;

pub struct SanitizedEvent {
    pub summary: Option<String>,
    pub dtstart: Option<String>,
    pub dtend: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
}

pub fn parse_vevent(raw: &str) -> Option<SanitizedEvent> {
    if !raw.contains("BEGIN:VEVENT") {
        return None;
    }

    let mut summary = None;
    let mut dtstart = None;
    let mut dtend = None;
    let mut location = None;
    let mut description = None;

    for line in raw.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("SUMMARY:") {
            summary = Some(sanitize_text(val, MAX_SUMMARY));
        } else if let Some(val) = line.strip_prefix("DTSTART:") {
            if is_valid_datetime(val) { dtstart = Some(val.to_string()); }
        } else if let Some(val) = line.strip_prefix("DTEND:") {
            if is_valid_datetime(val) { dtend = Some(val.to_string()); }
        } else if let Some(val) = line.strip_prefix("LOCATION:") {
            location = Some(sanitize_text(val, MAX_LOCATION));
        } else if let Some(val) = line.strip_prefix("DESCRIPTION:") {
            description = Some(sanitize_text(val, MAX_DESCRIPTION));
        }
        // All other properties (ATTACH, VALARM, URL, X-*, ATTENDEE, ORGANIZER)
        // are intentionally ignored/stripped.
    }

    Some(SanitizedEvent { summary, dtstart, dtend, location, description })
}

/// Generate a sanitized .ics file from parsed event fields.
pub fn to_ics(event: &SanitizedEvent) -> String {
    let mut lines = vec![
        "BEGIN:VCALENDAR".to_string(),
        "VERSION:2.0".to_string(),
        "PRODID:-//QRSnap//EN".to_string(),
        "BEGIN:VEVENT".to_string(),
    ];
    if let Some(s) = &event.summary { lines.push(format!("SUMMARY:{s}")); }
    if let Some(d) = &event.dtstart { lines.push(format!("DTSTART:{d}")); }
    if let Some(d) = &event.dtend { lines.push(format!("DTEND:{d}")); }
    if let Some(l) = &event.location { lines.push(format!("LOCATION:{l}")); }
    if let Some(d) = &event.description { lines.push(format!("DESCRIPTION:{d}")); }
    lines.push("END:VEVENT".to_string());
    lines.push("END:VCALENDAR".to_string());
    lines.join("\r\n")
}

/// Strip control characters and HTML tags, truncate to max_len.
fn sanitize_text(input: &str, max_len: usize) -> String {
    let mut out = String::with_capacity(input.len().min(max_len));
    let mut in_tag = false;
    for ch in input.chars() {
        if out.len() >= max_len { break; }
        if ch == '<' { in_tag = true; continue; }
        if ch == '>' { in_tag = false; continue; }
        if in_tag { continue; }
        if ch.is_control() && ch != '\n' { continue; }
        out.push(ch);
    }
    out
}

/// Basic validation: iCal datetime format (e.g. 20260315T100000Z)
fn is_valid_datetime(val: &str) -> bool {
    let val = val.trim();
    // Accept 8-digit date or 15-char datetime with optional Z/timezone
    val.len() >= 8
        && val.len() <= 20
        && val.chars().all(|c| c.is_ascii_alphanumeric() || c == 'T' || c == 'Z' || c == '-' || c == ':')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_event() {
        let raw = "BEGIN:VEVENT\nSUMMARY:Standup\nDTSTART:20260315T100000Z\nEND:VEVENT";
        let evt = parse_vevent(raw).unwrap();
        assert_eq!(evt.summary.as_deref(), Some("Standup"));
        assert_eq!(evt.dtstart.as_deref(), Some("20260315T100000Z"));
    }

    #[test]
    fn test_strips_attach_and_valarm() {
        let raw = "BEGIN:VEVENT\nSUMMARY:Test\nATTACH:https://evil.com\nVALARM:DISPLAY\nEND:VEVENT";
        let evt = parse_vevent(raw).unwrap();
        let ics = to_ics(&evt);
        assert!(!ics.contains("ATTACH"));
        assert!(!ics.contains("VALARM"));
    }

    #[test]
    fn test_strips_html_in_description() {
        let raw = "BEGIN:VEVENT\nDESCRIPTION:<script>alert(1)</script>Hello\nEND:VEVENT";
        let evt = parse_vevent(raw).unwrap();
        assert_eq!(evt.description.as_deref(), Some("alert(1)Hello"));
    }

    #[test]
    fn test_truncates_long_summary() {
        let long = "A".repeat(300);
        let raw = format!("BEGIN:VEVENT\nSUMMARY:{long}\nEND:VEVENT");
        let evt = parse_vevent(&raw).unwrap();
        assert_eq!(evt.summary.as_ref().unwrap().len(), 256);
    }

    #[test]
    fn test_invalid_datetime_skipped() {
        let raw = "BEGIN:VEVENT\nDTSTART:not-a-date-at-all!!\nEND:VEVENT";
        let evt = parse_vevent(raw).unwrap();
        assert!(evt.dtstart.is_none());
    }
}
