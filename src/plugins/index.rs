//! `/admin/plugins` — WP plugin-list parity.
//!
//! Rows come from two sources unioned: registered `PluginAdapter`s and
//! the imported `active_plugins` option. Adapters get a backend badge +
//! their screens; unadopted active plugins get an honest `—` badge and
//! no actions (visible, no fake controls).

use super::{adapters, Backend};
use crate::admin::{
    check_csrf, chrome, esc, html_page, need_auth, App, Sess, SiteCtx,
};
use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use rusqlite::Connection;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// One settings form field — keys map 1:1 onto the option names the
/// plugin used in wp_options (`settings` rows).
pub(crate) enum FieldKind {
    Text,
    Textarea,
    Select(&'static [(&'static str, &'static str)]),
    Checkbox,
    /// PHP-serialized value; round-trips via php_serialize on save.
    SerializedTextarea,
    /// A read-only note inside the form (no field, nothing posted).
    Note(&'static str),
}

pub(crate) struct FieldSpec {
    pub(crate) key: &'static str,
    pub(crate) label: &'static str,
    pub(crate) kind: FieldKind,
    pub(crate) help: &'static str,
}


#[derive(Deserialize)]
pub(crate) struct SettingsForm {
    csrf: String,
    #[serde(flatten)]
    pub(crate) fields: HashMap<String, String>,
}

/// Parse the imported `active_plugins` serialized option into plugin
/// dir slugs (`chaty/cht-icons.php` → `chaty`).
fn active_plugin_slugs(db: &Path) -> Vec<String> {
    let Ok(c) = crate::db::conn(db) else { return Vec::new() };
    let raw: String = c
        .query_row(
            "SELECT value FROM settings WHERE key='active_plugins'",
            [],
            |r| r.get(0),
        )
        .unwrap_or_default();
    if raw.is_empty() {
        return Vec::new();
    }
    let Ok(v) = crate::php_serialize::unserialize(raw.as_bytes()) else {
        return Vec::new();
    };
    let Some(arr) = v.as_array() else { return Vec::new() };
    arr.iter()
        .filter_map(|f| f.as_str())
        .filter_map(|f| f.split('/').next())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// `/admin/plugins` — the index page.
pub(crate) async fn index_page(State(app): State<App>, headers: HeaderMap) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if s.role != "admin" {
        return StatusCode::FORBIDDEN.into_response();
    }
    let body = plugins_index_html(&s, &sc);
    html_page(&s, "Plugins", &format!("{}{}", chrome("plugins", &s), body), true).into_response()
}

fn plugins_index_html(_s: &Sess, sc: &SiteCtx) -> String {
    let mut rows = String::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for a in adapters() {
        let d = a.descriptor();
        let slug = a.slug();
        seen.insert(slug.to_string());
        // A bridge adapter may name its WP plugin dir differently from
        // its rpress slug (goodlayers → goodlayers-core); mark that too
        // or the row duplicates as "no native adapter".
        if let Some(dir) = a.wp_plugin_file().split('/').next() {
            if !dir.is_empty() {
                seen.insert(dir.to_string());
            }
        }
        let (badge, actions) = match a.backend() {
            Backend::PhpBridge => {
                let open = a
                    .screens()
                    .first()
                    .map(|s| format!("/admin/plugins/{}/{}", slug, s.path))
                    .unwrap_or_else(|| format!("/admin/plugins/{slug}/"));
                // Bridge panel — workdir + wp.db state, same sources
                // `rpress bridge <site> status` reads.
                let state = sc.db.parent()
                    .map(|p| p.join("www/bridge").join(slug))
                    .and_then(|d| {
                        let db = d.join("wp.db");
                        let m = std::fs::metadata(&db).ok()?;
                        let age = m.modified().ok()
                            .and_then(|t| std::time::SystemTime::now().duration_since(t).ok())
                            .map(|d| format!("{}s", d.as_secs()))
                            .unwrap_or_else(|| "?".into());
                        Some(format!("{} ({} bytes, {} ago)", d.display(), m.len(), age))
                    })
                    .unwrap_or_else(|| "no workdir — builds on first request".to_string());
                (
                    format!("<span class=\"badge badge-bridge\">Bridge</span><div class=\"muted\" style=\"font-size:11px;margin-top:3px\">{}</div>", esc(&state)),
                    format!("<a href=\"{}\">Open</a>", esc(&open)),
                )
            }
            _ => {
                let has_settings = a.screens().iter().any(|s| s.path == "settings");
                let settings = if has_settings {
                    format!("<a href=\"/admin/plugins/{slug}/settings\">Settings</a>")
                } else {
                    String::new()
                };
                (
                    "<span class=\"badge badge-native\">Native</span>".to_string(),
                    settings,
                )
            }
        };
        rows.push_str(&format!(
            "<tr><td class=\"plugin-title column-primary\"><strong>{}</strong><div class=\"row-actions\">{}</div></td><td>{}</td><td>{}</td></tr>",
            esc(d.name),
            actions,
            esc(d.blurb),
            badge,
        ));
    }

    // Unadopted active plugins — the gap list, honestly rendered.
    for slug in active_plugin_slugs(&sc.db) {
        if seen.contains(&slug) {
            continue;
        }
        rows.push_str(&format!(
            "<tr class=\"plugin-inactive\"><td class=\"plugin-title column-primary\"><strong>{}</strong></td><td class=\"muted\">no native adapter yet</td><td><span class=\"badge\">—</span></td></tr>",
            esc(&slug),
        ));
    }

    format!(
        r#"<div class="wrap"><h1 class="wp-heading-inline">Plugins.</h1>
<hr class="wp-header-end">
<p class="muted" style="margin:4px 0 14px">Native plugins run inside rpress; Bridge rows proxy to the vendored WP runtime. Settings screens write the same option keys the plugin used in wp_options.</p>
<table class="wp-list-table widefat plugins striped"><thead><tr><th class="manage-column column-name column-primary">Plugin</th><th class="manage-column">Description</th><th class="manage-column">Backend</th></tr></thead><tbody>{rows}</tbody></table></div>
<style>
.badge{{display:inline-block;padding:2px 8px;border-radius:10px;font-size:11px;font-weight:600;background:var(--paper-2);color:var(--ink-2)}}
.badge-native{{background:oklch(88% 0.1 155);color:oklch(30% 0.12 160)}}
.badge-bridge{{background:oklch(92% 0.1 85);color:oklch(35% 0.13 60)}}
.plugin-inactive td{{opacity:.75}}
</style>"#
    )
}

/// Shared settings-screen save: `INSERT OR REPLACE` each posted key,
/// then `regen_site`. Screen POST handlers call `need_auth` first, then
/// this — it runs `check_csrf` itself.
pub(crate) fn save_settings_fields(s: &Sess, sc: &SiteCtx, f: &SettingsForm, spec: &[FieldSpec]) -> Result<(), Response> {
    check_csrf(s, &f.csrf)?;
    let c = crate::db::conn(&sc.db).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
    let tx = c.unchecked_transaction().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
    for fsp in spec {
        if let FieldKind::Note(_) = fsp.kind {
            continue;
        }
        let raw = f.fields.get(fsp.key).cloned().unwrap_or_default();
        let value = match &fsp.kind {
            FieldKind::Checkbox => {
                if raw.is_empty() { "0".to_string() } else { "1".to_string() }
            }
            FieldKind::SerializedTextarea => {
                // Structured editor: leaf inputs post as `key.a.b` (dotted
                // path from the blob root). Rebuild the Value, preserving
                // original scalar types, then re-serialize. Falls back to
                // the raw textarea when the blob isn't a map.
                let orig: String = tx
                    .query_row("SELECT value FROM settings WHERE key=?1", [fsp.key], |r| r.get(0))
                    .unwrap_or_default();
                let parsed = crate::php_serialize::unserialize(orig.as_bytes()).ok();
                let prefix = format!("{}.", fsp.key);
                let mut posted: Vec<(String, &str)> = f
                    .fields
                    .iter()
                    .filter_map(|(k, v)| {
                        k.strip_prefix(&prefix).map(|p| (p.to_string(), v.as_str()))
                    })
                    .collect();
                posted.sort_by(|a, b| a.0.cmp(&b.0));
                match (&parsed, posted.is_empty()) {
                    (Some(v @ serde_json::Value::Object(_)), false) => {
                        let mut new_v = v.clone();
                        // Unchecked checkboxes post nothing — fill "0" for
                        // every bool leaf so absence means false.
                        for (path, _) in bool_leaves(&new_v, fsp.key) {
                            // bool_leaves paths carry the `key.` prefix;
                            // posted entries are already stripped.
                            let rel = path.strip_prefix(&prefix).unwrap_or(&path);
                            if !posted.iter().any(|(p, _)| p == rel) {
                                posted.push((rel.to_string(), "0"));
                            }
                        }
                        for (path, val) in &posted {
                            set_leaf(&mut new_v, path, val);
                        }
                        String::from_utf8_lossy(&crate::php_serialize::serialize(&new_v)).to_string()
                    }
                    (Some(_), true) if raw.trim().is_empty() => {
                        String::from_utf8_lossy(&crate::php_serialize::serialize(parsed.as_ref().unwrap())).to_string()
                    }
                    _ => {
                        if raw.trim().is_empty() {
                            String::new()
                        } else {
                            match crate::php_serialize::unserialize(raw.as_bytes())
                                .map(|v| crate::php_serialize::serialize(&v))
                            {
                                Ok(b) => String::from_utf8_lossy(&b).to_string(),
                                Err(_) => {
                                    return Err((
                                        StatusCode::BAD_REQUEST,
                                        format!("invalid serialized value for {}", fsp.key),
                                    )
                                        .into_response())
                                }
                            }
                        }
                    }
                }
            }
            _ => raw,
        };
        tx.execute(
            "INSERT OR REPLACE INTO settings(key, value) VALUES (?1, ?2)",
            rusqlite::params![fsp.key, value],
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
    }
    tx.commit().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
    crate::admin::admin::regen_site(sc);
    Ok(())
}

/// Render a settings form from `FieldSpec`s — values pulled live from
/// `settings`.
pub(crate) fn settings_form_html(
    conn: &Connection,
    action: &str,
    csrf: &str,
    title: &str,
    spec: &[FieldSpec],
) -> String {
    let mut rows = String::new();
    for f in spec {
        if let FieldKind::Note(text) = &f.kind {
            rows.push_str(&format!("<tr><th></th><td><p class=\"description\">{text}</p></td></tr>"));
            continue;
        }
        let cur: String = conn
            .query_row("SELECT value FROM settings WHERE key=?1", [f.key], |r| r.get(0))
            .unwrap_or_default();
        let field = match &f.kind {
            FieldKind::Text => format!(
                "<input type=\"text\" name=\"{k}\" id=\"{k}\" value=\"{v}\" class=\"regular-text\">",
                k = f.key,
                v = esc(&cur)
            ),
            FieldKind::Textarea => format!(
                "<textarea name=\"{k}\" id=\"{k}\" rows=\"5\" class=\"large-text code\">{v}</textarea>",
                k = f.key,
                v = esc(&cur)
            ),
            FieldKind::SerializedTextarea => {
                // Parse the blob into real inputs (nested maps →
                // fieldsets); falls back to the raw textarea when the
                // value isn't a map or fails to parse.
                match crate::php_serialize::unserialize(cur.as_bytes()) {
                    Ok(v @ serde_json::Value::Object(_)) => {
                        structured_fields_html(f.key, &v)
                    }
                    _ => format!(
                        "<textarea name=\"{k}\" id=\"{k}\" rows=\"6\" class=\"large-text code\">{v}</textarea><p class=\"description\">PHP-serialized — save validates the round-trip.</p>",
                        k = f.key,
                        v = esc(&cur)
                    ),
                }
            }
            FieldKind::Select(opts) => {
                let mut o = String::new();
                for (val, lab) in *opts {
                    let sel = if *val == cur { " selected" } else { "" };
                    o.push_str(&format!("<option value=\"{val}\"{sel}>{lab}</option>"));
                }
                format!("<select name=\"{k}\" id=\"{k}\">{o}</select>", k = f.key)
            }
            FieldKind::Checkbox => {
                let chk = if cur == "1" { " checked" } else { "" };
                format!("<input type=\"checkbox\" name=\"{k}\" id=\"{k}\" value=\"1\"{chk}>", k = f.key)
            }
            FieldKind::Note(_) => unreachable!(),
        };
        let help = if f.help.is_empty() {
            String::new()
        } else {
            format!("<p class=\"description\">{}</p>", esc(f.help))
        };
        rows.push_str(&format!(
            "<tr><th scope=\"row\"><label for=\"{k}\">{l}</label></th><td>{field}{help}</td></tr>",
            k = f.key,
            l = esc(f.label)
        ));
    }
    format!(
        r#"<h1>{title}</h1><hr class="wp-header-end">
<form method="post" action="{action}">
<input type="hidden" name="csrf" value="{csrf}">
<table class="form-table" role="presentation"><tbody>{rows}</tbody></table>
<p class="submit"><button type="submit" class="button button-primary">Save Changes</button></p>
</form>"#
    )
}

/// Full-page settings screen — `chrome` + form + `chrome_end`. GET
/// handlers return this; POST handlers call `save_settings_fields` then
/// redirect back (WP parity).
pub(crate) fn settings_page_html(
    s: &Sess,
    sc: &SiteCtx,
    action: &str,
    title: &str,
    spec: &[FieldSpec],
) -> Response {
    match crate::db::conn(&sc.db) {
        Ok(conn) => html_page(
            s,
            title,
            &format!(
                "{}<div class=\"wrap\">{}</div>",
                chrome(title, s),
                settings_form_html(&conn, action, &s.csrf, title, spec)
            ),
            false,
        )
        .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// A serialized `Option` map → nested plain inputs: bools → checkboxes,
/// numbers → number inputs, strings → text, arrays of scalars → one
/// comma-joined input, nested maps → fieldsets named `parent.child`.
/// Input names are dotted paths (`key.a.b`) — `save_settings_fields`
/// walks them back into the `Value` before re-serializing.
fn structured_fields_html(root: &str, v: &serde_json::Value) -> String {
    let mut out = String::from(
        "<div class=\"structured-settings\" style=\"display:flex;flex-direction:column;gap:8px\">",
    );
    structured_walk(&mut out, root, v, 0);
    out.push_str("</div>");
    out
}

fn structured_walk(out: &mut String, path: &str, v: &serde_json::Value, depth: usize) {
    match v {
        serde_json::Value::Object(m) => {
            if depth > 0 {
                out.push_str(&format!(
                    "<fieldset style=\"border:1px solid var(--border);border-radius:6px;padding:10px 12px;margin:4px 0\">\
                     <legend style=\"font-weight:600;padding:0 6px\">{}</legend>",
                    esc(path.rsplit('.').next().unwrap_or(path))
                ));
            }
            for (k, val) in m {
                let p = format!("{path}.{k}");
                if val.is_object() {
                    structured_walk(out, &p, val, depth + 1);
                } else {
                    out.push_str(&format!(
                        "<label style=\"display:flex;align-items:baseline;gap:10px\">\
                         <span style=\"min-width:170px;color:var(--muted);font-size:12px\" title=\"{p}\">{label}</span>{input}</label>",
                        p = esc(&p),
                        label = esc(k),
                        input = leaf_input(&p, val)
                    ));
                }
            }
            if depth > 0 {
                out.push_str("</fieldset>");
            }
        }
        _ => out.push_str(&leaf_input(path, v)),
    }
}

fn leaf_input(path: &str, v: &serde_json::Value) -> String {
    let name = esc(path);
    match v {
        serde_json::Value::Bool(b) => {
            let chk = if *b { " checked" } else { "" };
            format!("<input type=\"checkbox\" name=\"{name}\" value=\"1\"{chk}>")
        }
        serde_json::Value::Number(n) => format!(
            "<input type=\"text\" name=\"{name}\" value=\"{}\" class=\"small-text\">",
            n
        ),
        serde_json::Value::String(s) => {
            let cls = if s.len() > 60 { "large-text" } else { "regular-text" };
            format!("<input type=\"text\" name=\"{name}\" value=\"{}\" class=\"{cls}\">", esc(s))
        }
        serde_json::Value::Array(items) => {
            // Scalar lists edit as comma-separated text.
            let joined = items
                .iter()
                .map(|i| match i {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "<input type=\"text\" name=\"{name}\" value=\"{}\" class=\"large-text\"><p class=\"description\">comma-separated</p>",
                esc(&joined)
            )
        }
        serde_json::Value::Null => {
            format!("<input type=\"text\" name=\"{name}\" value=\"\" class=\"regular-text\">")
        }
        serde_json::Value::Object(_) => String::new(), // handled by walk
    }
}

/// Every `path.leaf` under `root` whose current value is a bool — used on
/// save to inject `0` for checkboxes the browser didn't post.
fn bool_leaves(v: &serde_json::Value, root: &str) -> Vec<(String, ())> {
    let mut out = Vec::new();
    walk_leaves(v, root, &mut |path, leaf| {
        if leaf.is_boolean() {
            out.push((path.to_string(), ()));
        }
    });
    out
}

fn walk_leaves<'a>(v: &'a serde_json::Value, path: &str, f: &mut impl FnMut(&str, &'a serde_json::Value)) {
    match v {
        serde_json::Value::Object(m) => {
            for (k, val) in m {
                walk_leaves(val, &format!("{path}.{k}"), f);
            }
        }
        _ => f(path, v),
    }
}

/// Set `v` at dotted `path`, coercing the posted string back to the
/// original leaf's type (bool/number/array preserved, strings stay
/// strings). Unknown paths are ignored — the blob is the schema.
fn set_leaf(v: &mut serde_json::Value, path: &str, val: &str) {
    // `path` is relative to the blob root — the posted `key.a.b` name
    // arrives already stripped to `a.b`.
    let segs: Vec<&str> = path.split('.').collect();
    set_in(v, &segs, val);
}

fn set_in(v: &mut serde_json::Value, segs: &[&str], val: &str) {
    if segs.is_empty() {
        return;
    }
    if let serde_json::Value::Object(m) = v {
        if segs.len() == 1 {
            if let Some(slot) = m.get_mut(segs[0]) {
                *slot = coerce(slot, val);
            }
            return;
        }
        if let Some(next) = m.get_mut(segs[0]) {
            set_in(next, &segs[1..], val);
        }
    }
}

fn coerce(orig: &serde_json::Value, raw: &str) -> serde_json::Value {
    match orig {
        serde_json::Value::Bool(_) => serde_json::Value::Bool(raw == "1" || raw == "true"),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                raw.trim().parse::<i64>().map(serde_json::Value::from).unwrap_or_else(|_| orig.clone())
            } else {
                raw.trim().parse::<f64>().map(serde_json::Value::from).unwrap_or_else(|_| orig.clone())
            }
        }
        serde_json::Value::Array(_) => serde_json::Value::Array(
            raw.split(',')
                .map(|s| serde_json::Value::String(s.trim().to_string()))
                .filter(|s| !s.as_str().unwrap_or("").is_empty())
                .collect(),
        ),
        _ => serde_json::Value::String(raw.to_string()),
    }
}
