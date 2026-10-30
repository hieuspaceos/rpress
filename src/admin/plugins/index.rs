//! `/admin/plugins` — WP plugin-list parity.
//!
//! Rows come from two sources unioned: registered `PluginAdapter`s and
//! the imported `active_plugins` option. Adapters get a backend badge +
//! their screens; unadopted active plugins get an honest `—` badge and
//! no actions (visible, no fake controls).

use super::{adapters, Backend};
use crate::admin::{
    check_csrf, chrome, chrome_end, esc, need_auth, App, Sess, SiteCtx,
};
use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
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

impl FieldSpec {
    pub(crate) fn text(key: &'static str, label: &'static str) -> Self {
        FieldSpec { key, label, kind: FieldKind::Text, help: "" }
    }
    pub(crate) fn checkbox(key: &'static str, label: &'static str) -> Self {
        FieldSpec { key, label, kind: FieldKind::Checkbox, help: "" }
    }
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
    Html(format!("{}{}", chrome("plugins", &s), body)).into_response()
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
                (
                    "<span class=\"badge badge-bridge\">Bridge</span>".to_string(),
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
        r#"<h1 class="wp-heading-inline">Plugins.</h1>
<hr class="wp-header-end">
<p class="muted" style="margin:4px 0 14px">Native plugins run inside rpress; Bridge rows proxy to the vendored WP runtime. Settings screens write the same option keys the plugin used in wp_options.</p>
<table class="wp-list-table widefat plugins striped"><thead><tr><th class="manage-column column-name column-primary">Plugin</th><th class="manage-column">Description</th><th class="manage-column">Backend</th></tr></thead><tbody>{rows}</tbody></table>
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
                // Round-trip guard — refuse garbage instead of storing it.
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
            FieldKind::SerializedTextarea => format!(
                "<textarea name=\"{k}\" id=\"{k}\" rows=\"6\" class=\"large-text code\">{v}</textarea><p class=\"description\">PHP-serialized — save validates the round-trip.</p>",
                k = f.key,
                v = esc(&cur)
            ),
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
        Ok(conn) => Html(format!(
            "{}{}{}",
            chrome(title, s),
            settings_form_html(&conn, action, &s.csrf, title, spec),
            chrome_end(s)
        ))
        .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
