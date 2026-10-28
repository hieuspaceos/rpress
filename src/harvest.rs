//! Origin-page harvesting shared by generate (fetch-at-generate for
//! `harvest_posts` elementor ids) and import (fetch-at-import for spec
//! `harvest = true` kinds → `harvested_pages`).
//!
//! The fetched page is the origin's own rendered output — fidelity by
//! construction. Origin URLs are rewritten to a placeholder target:
//! callers pass the local `site_url` (generate) or `{{ site_url }}`
//! (import-time storage, resolved by `rewrite_origin` at render).

use anyhow::{Context, Result};
use std::process::Command;

/// Fetch `origin + path` and rewrite every absolute origin URL to
/// `target` (the caller's site_url or the `{{ site_url }}` placeholder).
/// Also rewrites CF7 form actions so harvested enquiry forms post to
/// `/form/enquiry` (the intake endpoint) instead of a URL rpress never
/// serves — a visibly broken form is worse than a mapped one.
pub(crate) fn page(origin: &str, path: &str, target: &str) -> Result<String> {
    let base = origin.trim_end_matches('/');
    // Scheme-less stored origins need a scheme to fetch; default https.
    let base = if base.contains("://") {
        base.to_string()
    } else {
        format!("https://{base}")
    };
    let base = base.as_str();
    let url = format!("{base}/{}", path.trim_start_matches('/'));
    let o = Command::new("curl")
        .args(["-sfL", "--max-time", "30", "-A", "rpress-harvest/1.0", &url])
        .output()
        .with_context(|| format!("curl {url}"))?;
    if !o.status.success() {
        anyhow::bail!("{url} → http {}", o.status);
    }
    let mut html = String::from_utf8_lossy(&o.stdout).into_owned();
    if html.len() < 500 {
        anyhow::bail!("{url} too short ({}b) — refusing", html.len());
    }
    let local = target.trim_end_matches('/');
    html = html.replace(base, local);
    html = html.replace(&base.replace('/', "\\/"), &local.replace('/', "\\/"));
    html = rewrite_cf7_actions(&html, local, path);
    Ok(html)
}

/// CF7 renders `<form action="{page_url}#wpcf7-f{id}-o{n}">`; after the
/// origin→local rewrite that still lands on a static page URL, so a
/// submit would POST to a 404. Point CF7 forms at the `/form/enquiry`
/// intake endpoint instead and preserve the return path + post slug as
/// hidden fields (same convention as `babe_enquiry.html`). A form is
/// "CF7" when its tag or wrapper carries the wpcf7 marker AND the action
/// fragment does — a deliberate `#wpcf7` anchor check keeps non-CF7
/// forms with their own endpoints untouched.
fn rewrite_cf7_actions(html: &str, local: &str, path: &str) -> String {
    if !html.contains("wpcf7") {
        return html.to_string();
    }
    // Return path + slug for _page/tour hidden fields: `path` is the
    // page's rel dir ("booking/foo/" or "es/booking/foo/").
    let rel = format!("/{}", path.trim_end_matches('/'));
    let slug = path
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_string();
    let inject = format!(
        "<input type=\"hidden\" name=\"_page\" value=\"{rel}\" />\
         <input type=\"hidden\" name=\"tour\" value=\"{slug}\" />"
    );
    // Intentionally loose scan below: prefix-matches `<form-x`/
    // `<formdata` and is case-sensitive — CF7 origin output is always
    // lowercase `<form ` so a loose match is harmless in practice.
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(fi) = rest.find("<form") {
        out.push_str(&rest[..fi]);
        rest = &rest[fi..];
        let Some(end) = rest.find('>') else {
            out.push_str(rest);
            return out;
        };
        let tag = &rest[..end];
        let newtag = rewrite_tag_action(tag, local);
        let is_cf7 = tag.contains("wpcf7") || newtag != tag;
        out.push_str(&newtag);
        rest = &rest[end..];
        if is_cf7 {
            // rest begins with '>': emit it, then the hidden fields.
            out.push('>');
            out.push_str(&inject);
            rest = &rest[1..];
        }
    }
    out.push_str(rest);
    out
}

/// `action="…#wpcf7-…"` → `{local}/form/enquiry` inside one <form> tag.
/// Returns the tag unchanged when no CF7-fragment action is present.
fn rewrite_tag_action(tag: &str, local: &str) -> String {
    let Some(ai) = tag.find("action=") else {
        return tag.to_string();
    };
    let after = ai + 7;
    let bytes = tag.as_bytes();
    let (vstart, vend) = match bytes.get(after) {
        Some(b'"') | Some(b'\'') => {
            let q = bytes[after] as char;
            match tag[after + 1..].find(q) {
                Some(e) => (after + 1, after + 1 + e),
                None => return tag.to_string(),
            }
        }
        _ => {
            // Unquoted value: ends at whitespace or the tag's '>' —
            // `tag` excludes '>' so a missing-whitespace value is the
            // tag tail.
            match tag[after..].find(char::is_whitespace) {
                Some(e) => (after, after + e),
                None => (after, tag.len()),
            }
        }
    };
    if !tag[vstart..vend].contains("wpcf7") {
        return tag.to_string();
    }
    let mut t = tag.to_string();
    t.replace_range(vstart..vend, &format!("{local}/form/enquiry"));
    t
}
