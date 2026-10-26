//! CF7-style tag parser — subset only. Parses `[type* name opt:val ...]`
//! tags inside form markup. Used by:
//!   - `forms_editor.rs` to validate markup on save (reject unknown types,
//!     missing names, unclosed brackets, `[name]` mail refs that don't exist)
//!   - `intake.rs` to enforce required fields + type-check on submission.
//!
//! Supported types: text, email, url, tel, number, date, textarea, select,
//! checkbox, radio, acceptance, quiz, file, submit.

use std::collections::HashMap;

#[derive(Debug, Clone)]
#[allow(dead_code)] // opts/at retained for error reporting completeness
pub struct Tag {
    /// `text` / `email` / `textarea` / ...
    pub type_: String,
    /// Field name (`your-name`, `rating`, ...). `submit` has an empty name.
    pub name: String,
    /// Whether `*` was present.
    pub required: bool,
    /// Raw options: `min:1`, `max:5`, `id:foo`, `class:bar`, quoted
    /// strings for `select`/labels.
    pub opts: Vec<String>,
    /// Byte offset in source (for error reporting).
    pub at: usize,
}

const KNOWN: &[&str] = &[
    "text", "email", "url", "tel", "number", "date", "textarea", "select",
    "checkbox", "radio", "acceptance", "quiz", "file", "submit",
];

/// Parse all `[...]` tags in markup. Returns `Err(msg)` on the first
/// malformed or unknown tag. Bare `[` and `]` inside text are tolerated
/// (skipped) — CF7 is forgiving there too.
pub fn parse(markup: &str) -> Result<Vec<Tag>, String> {
    let b = markup.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'[' {
            i += 1;
            continue;
        }
        // find matching ']' (no nesting in CF7)
        let mut j = i + 1;
        while j < b.len() && b[j] != b']' {
            // '[' before ']' → the first '[' was text
            if b[j] == b'[' {
                break;
            }
            j += 1;
        }
        if j >= b.len() || b[j] != b']' {
            i += 1; // treat as text
            continue;
        }
        let inner = &markup[i + 1..j];
        if let Some(tag) = parse_tag(inner).map_err(|e| format!("{e} (at byte {i})"))? {
            out.push(Tag { at: i, ..tag });
        }
        i = j + 1;
    }
    Ok(out)
}

fn parse_tag(inner: &str) -> Result<Option<Tag>, String> {
    let s = inner.trim();
    if s.is_empty() {
        return Ok(None);
    }
    let mut it = s.split_whitespace();
    let mut typ = it.next().unwrap_or("");
    if typ.is_empty() {
        return Ok(None);
    }
    // CF7 types may carry a trailing `*` — e.g. `text*`
    let required = typ.ends_with('*');
    if required {
        typ = &typ[..typ.len() - 1];
    }
    if !KNOWN.contains(&typ) {
        return Err(format!("unknown tag type '{typ}'"));
    }
    // Collect opts. `submit` may have just a label; others need a name.
    let mut opts: Vec<String> = Vec::new();
    let mut name = String::new();
    let mut first = true;
    for tok in it {
        if first && !tok.contains(':') && !tok.starts_with('"') {
            name = tok.to_string();
            first = false;
            continue;
        }
        first = false;
        opts.push(tok.to_string());
    }
    if typ != "submit" && name.is_empty() {
        return Err(format!("tag '{typ}' missing name"));
    }
    Ok(Some(Tag {
        type_: typ.to_string(),
        name,
        required,
        opts,
        at: 0,
    }))
}

/// Names of all non-submit fields — used to validate `[name]` mail refs.
pub fn field_names(tags: &[Tag]) -> Vec<&str> {
    tags.iter()
        .filter(|t| t.type_ != "submit")
        .map(|t| t.name.as_str())
        .collect()
}

/// Validate markup + mail templates together. Returns the first error.
pub fn validate(markup: &str, mail_to: &str, mail_from: &str, mail_subject: &str, mail_body: &str) -> Result<(), String> {
    let tags = parse(markup)?;
    if tags.iter().all(|t| t.type_ != "submit") {
        return Err("form has no [submit] tag".into());
    }
    let names: Vec<&str> = field_names(&tags);
    let check_refs = |tpl: &str, what: &str| -> Result<(), String> {
        for m in tpl.match_indices('[') {
            if let Some(end) = tpl[m.0..].find(']') {
                let name = &tpl[m.0 + 1..m.0 + end];
                if !name.is_empty() && !names.contains(&name) {
                    return Err(format!("{what} references [{name}] which is not a field"));
                }
            }
        }
        Ok(())
    };
    check_refs(mail_subject, "mail subject")?;
    check_refs(mail_body, "mail body")?;
    check_refs(mail_to, "mail to")?;
    check_refs(mail_from, "mail from")?;
    Ok(())
}

/// Required fields by name — used at intake time.
#[allow(dead_code)] // intake-validation parity; retained as library surface
pub fn required_names(tags: &[Tag]) -> Vec<&str> {
    tags.iter()
        .filter(|t| t.required && !t.name.is_empty())
        .map(|t| t.name.as_str())
        .collect()
}

/// Light type-check for a submitted value — returns an error key like
/// `email_invalid` when the value looks wrong. Only checks `email`, `url`,
/// `tel`, `number`, `date` (others accept anything).
pub fn check_value(type_: &str, v: &str) -> Option<&'static str> {
    if v.is_empty() {
        return None; // required-ness checked elsewhere
    }
    match type_ {
        "email" => {
            if v.contains('@') && v.split('@').nth(1).map(|d| d.contains('.')).unwrap_or(false) {
                None
            } else {
                Some("email_invalid")
            }
        }
        "url" => {
            if v.starts_with("http://") || v.starts_with("https://") {
                None
            } else {
                Some("url_invalid")
            }
        }
        "number" => {
            if v.parse::<f64>().is_ok() {
                None
            } else {
                Some("number_invalid")
            }
        }
        "date" => {
            let ok = v.len() == 10
                && v.chars().nth(4) == Some('-')
                && v.chars().nth(7) == Some('-');
            if ok {
                None
            } else {
                Some("date_invalid")
            }
        }
        "tel" => {
            if v.chars().all(|c| c.is_ascii_digit() || "+-() ".contains(c)) {
                None
            } else {
                Some("tel_invalid")
            }
        }
        _ => None,
    }
}

/// Substitute `[name]` placeholders in a template with submitted values.
#[allow(dead_code)] // CF7 mail-body templating parity; retained as library surface
pub fn substitute(tpl: &str, values: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(tpl.len());
    let mut rest = tpl;
    while let Some(i) = rest.find('[') {
        out.push_str(&rest[..i]);
        if let Some(j) = rest[i..].find(']') {
            let name = &rest[i + 1..i + j];
            if let Some(v) = values.get(name) {
                out.push_str(v);
            } else {
                out.push_str(&rest[i..i + j + 1]);
            }
            rest = &rest[i + j + 1..];
        } else {
            out.push_str(&rest[i..]);
            break;
        }
    }
    out.push_str(rest);
    out
}
