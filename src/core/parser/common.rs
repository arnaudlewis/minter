// ── Utility functions ───────────────────────────────────

pub(crate) fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or(s)
}

pub(crate) fn parse_quoted_string(s: &str) -> Option<String> {
    let s = s.trim();
    if !s.starts_with('"') {
        return None;
    }
    let rest = &s[1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

pub(crate) fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}
