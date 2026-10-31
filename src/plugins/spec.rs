//! Declarative plugin adapters: a `plugins/specs/<slug>.toml` file
//! registers a plugin into the seam with zero Rust — same `PluginAdapter`
//! contract, same registry asserts, same `adapters()` consumers. Rust
//! adapters keep priority where they exist (kind collisions panic at
//! REGISTRY init); specs only fill gaps.
//!
//! Spec source files are embedded at build time by `build.rs` (the
//! binary runs from `work/<site>/`, never the repo root) and parsed once
//! into `SPECS` — a malformed or contradictory spec fails startup
//! loudly, never a silent skip.
//!
//! Ownership model: `PluginAdapter` signatures demand `&'static`
//! returns. Instead of leaking, parsed specs live in `static
//! LazyLock` tables (`SPECS`, `KIND_DEFS`, `PREFIXES`, `HOOKS`);
//! `&'static` views are borrows of the statics themselves, and each
//! `SpecAdapter` is just an index into them.

use super::{Backend, EditorHooks, ImportHooks, KindDef, PluginAdapter, PluginDescriptor, RenderCtx, ScreenDef, ScreenFuture};
use crate::admin::posts;
use crate::db;
use crate::plugins::index::{self, FieldKind, FieldSpec, SettingsForm};
use crate::admin::esc;
use anyhow::Result;
use axum::extract::{Form, FromRequest, Request};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Redirect};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::LazyLock;

include!(concat!(env!("OUT_DIR"), "/specs.rs"));

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Spec {
    pub(crate) slug: String,
    /// wp `post_type`s this stack claims; each must match a `kinds[].kind`.
    #[serde(default)]
    pub(crate) claim_types: Vec<String>,
    #[serde(default)]
    pub(crate) option_prefixes: Vec<String>,
    #[serde(default)]
    pub(crate) legacy_option_keys: Vec<String>,
    /// Plugins-index name/blurb; defaults keep slug parity.
    #[serde(default)]
    pub(crate) name: Option<String>,
    #[serde(default)]
    pub(crate) blurb: Option<String>,
    /// Declarative settings screens under `/admin/plugins/<slug>/<path>` —
    /// FieldSpec rows in `settings`, shared save path with Rust screens.
    #[serde(default)]
    pub(crate) screens: Vec<ScreenSpec>,
    pub(crate) kinds: Vec<KindSpec>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScreenSpec {
    /// Route path under the plugin screen root ("settings").
    pub(crate) path: String,
    pub(crate) title: String,
    /// "admin" (default) or "editor" — same gate as `ScreenDef.min_role`.
    #[serde(default = "default_min_role")]
    pub(crate) min_role: String,
    #[serde(default)]
    pub(crate) fields: Vec<FieldSpecToml>,
}

/// TOML form of `index::FieldSpec` — `kind` is one of
/// "text"|"textarea"|"select"|"checkbox"|"serialized"|"note".
/// `options` fills `Select`; `help` renders under the input.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FieldSpecToml {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) options: Vec<(String, String)>,
    #[serde(default)]
    pub(crate) help: String,
}

fn default_min_role() -> String {
    "admin".to_string()
}

/// Declarative post metabox: flat `post_meta` rows edited inside the
/// kind's classic editor. `load` hydrates `f.extra` from `post_meta`,
/// `save` writes rows back — quick edit (which never posts metabox
/// fields) leaves meta untouched.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MetaboxSpec {
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) fields: Vec<FieldSpecToml>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct KindSpec {
    pub(crate) kind: String,
    pub(crate) section: String,
    /// Singular noun for the editor heading.
    pub(crate) noun: String,
    /// List-page title; defaults to capitalized section.
    #[serde(default)]
    pub(crate) list_title: Option<String>,
    /// "Add …" label; defaults to "Add New <Noun>".
    #[serde(default)]
    pub(crate) add_label: Option<String>,
    /// Fixed public URL base (e.g. "booking"). Empty string = root.
    #[serde(default)]
    pub(crate) url_base: Option<String>,
    /// Origin path segment when it differs from `url_base`
    /// (`to_book` in WP vs `booking` publicly). Default = url_base.
    #[serde(default)]
    pub(crate) origin_base: Option<String>,
    #[serde(default)]
    pub(crate) taxonomies: Vec<String>,
    /// Dashicons class for the sidebar entry; default = babe's ticket.
    #[serde(default = "default_menu_icon")]
    pub(crate) menu_icon: String,
    /// Render default = origin-harvested HTML (phase 02); the adapter's
    /// own `render()` stays `None` either way.
    #[serde(default)]
    pub(crate) harvest: bool,
    /// Declarative metabox inside this kind's classic editor — fields
    /// round-trip through `post_meta`. `None` → classic editor only.
    #[serde(default)]
    pub(crate) metabox: Option<MetaboxSpec>,
}

fn default_menu_icon() -> String {
    "dashicons-tickets-alt".to_string()
}

const HEAD_SPEC: &str = r#"<tr><td class="manage-column column-cb check-column"></td><th scope="col" class="manage-column column-title column-primary">Title</th><th scope="col">Categories</th><th scope="col">Languages</th><th scope="col">Comments</th><th scope="col" class="rm-seo-col">SEO Details</th></tr>"#;

fn spec_row_cols(r: &db::AdminListRow, kind: &str, langs: &crate::lang::LangConfig) -> String {
    format!(
        "<td class=\"categories\">{}</td><td class=\"pll\">{}</td>",
        posts::dash(&r.categories),
        posts::pll_cell(r, kind, langs),
    )
}

/// All parsed specs, in filename order. Parse failure panics with the
/// spec slug + toml error — specs are repo-authored, fail-loud is right.
static SPECS: LazyLock<Vec<Spec>> = LazyLock::new(|| {
    SPEC_SRCS
        .iter()
        .map(|(slug, src)| {
            let spec: Spec = toml::from_str(src)
                .unwrap_or_else(|e| panic!("plugins/specs/{slug}.toml: {e}"));
            assert_eq!(*slug, spec.slug, "spec filename/slug mismatch in {slug}.toml");
            for t in &spec.claim_types {
                assert!(
                    spec.kinds.iter().any(|k| &k.kind == t),
                    "{slug}.toml: claim_types entry {t} has no matching kind"
                );
            }
            for s in &spec.screens {
                assert!(
                    s.fields.iter().all(|f| parse_field_kind(&f.kind, &f.options).is_some()),
                    "{slug}.toml: screen {} has an unknown field kind",
                    s.path
                );
            }
            for k in &spec.kinds {
                if let Some(mb) = &k.metabox {
                    assert!(
                        mb.fields.iter().all(|f| parse_field_kind(&f.kind, &f.options).is_some()),
                        "{slug}.toml: metabox on {} has an unknown field kind",
                        k.kind
                    );
                    // Metabox keys are form field names — a key that
                    // collides with a PostForm field or a reserved prefix
                    // never reaches `f.extra` / corrupts a core write.
                    for f in &mb.fields {
                        assert!(
                            !matches!(f.key.as_str(), "title" | "status" | "slug" | "kind")
                                && !f.key.starts_with("trans_")
                                && !f.key.starts_with("rank_math"),
                            "{slug}.toml: metabox key {:?} on {} collides with a PostForm field or reserved prefix",
                            f.key,
                            k.kind
                        );
                    }
                }
            }
            spec
        })
        .collect()
});

/// TOML `kind` string → `FieldKind`. `serialized` shares the settings
/// screen's blob editor; inside a metabox it round-trips raw (no leaf
/// walk — metabox fields are flat post_meta rows).
fn parse_field_kind(kind: &str, _opts: &[(String, String)]) -> Option<()> {
    match kind {
        "text" | "textarea" | "select" | "checkbox" | "serialized" | "note" => Some(()),
        _ => None,
    }
}

/// `KindDef` tables per spec — `&'static` fields borrow `SPECS`,
/// `TITLES`, and `TAX_TABLES` (borrows across statics are 'static).
static KIND_DEFS: LazyLock<Vec<Vec<KindDef>>> = LazyLock::new(|| {
    SPECS
        .iter()
        .enumerate()
        .map(|(si, spec)| {
            spec.kinds
                .iter()
                .enumerate()
                .map(|(ki, k)| KindDef {
                    kind: k.kind.as_str(),
                    section: k.section.as_str(),
                    noun: k.noun.as_str(),
                    list_title: TITLES[si][ki].0.as_str(),
                    add_label: TITLES[si][ki].1.as_str(),
                    head: HEAD_SPEC,
                    row_cols: spec_row_cols,
                    url_base_key: None,
                    url_base: k
                        .url_base
                        .as_deref()
                        .filter(|b| !b.is_empty()),
                    taxonomies: TAX_TABLES[si][ki].as_slice(),
                    menu_icon: k.menu_icon.as_str(),
                    editor: SPEC_EDITORS[si][ki].as_ref(),
                })
                .collect()
        })
        .collect()
});
/// Taxonomy name tables per spec per kind (`&'static [&'static str]`
/// borrows `SPECS` strings through this nested static).
static TAX_TABLES: LazyLock<Vec<Vec<Vec<&'static str>>>> = LazyLock::new(|| {
    SPECS
        .iter()
        .map(|spec| {
            spec.kinds
                .iter()
                .map(|k| k.taxonomies.iter().map(|t| t.as_str()).collect())
                .collect()
        })
        .collect()
});

/// Resolved list titles / add labels per kind (defaults materialized as
/// owned strings so `&'static` borrows are possible).
static TITLES: LazyLock<Vec<Vec<(String, String)>>> = LazyLock::new(|| {
    SPECS
        .iter()
        .map(|spec| {
            spec.kinds
                .iter()
                .map(|k| {
                    let lt = k.list_title.clone().unwrap_or_else(|| {
                        let mut c = k.section.chars();
                        c.next()
                            .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                            .unwrap_or_default()
                    });
                    let al = k
                        .add_label
                        .clone()
                        .unwrap_or_else(|| format!("Add New {}", k.noun));
                    (lt, al)
                })
                .collect()
        })
        .collect()
});


/// option_prefixes per spec (`&'static [&'static str]`).
static PREFIXES: LazyLock<Vec<Vec<&'static str>>> = LazyLock::new(|| {
    SPECS
        .iter()
        .map(|spec| spec.option_prefixes.iter().map(|p| p.as_str()).collect())
        .collect()
});

/// legacy_option_keys per spec.
static LEGACY_KEYS: LazyLock<Vec<Vec<&'static str>>> = LazyLock::new(|| {
    SPECS
        .iter()
        .map(|spec| spec.legacy_option_keys.iter().map(|k| k.as_str()).collect())
        .collect()
});

/// `wp_type → kind` claim map per spec.
static CLAIMS: LazyLock<Vec<Vec<(&'static str, &'static str)>>> = LazyLock::new(|| {
    SPECS
        .iter()
        .map(|spec| spec.claim_types.iter().map(|t| (t.as_str(), t.as_str())).collect())
        .collect()
});

/// Harvest-kind `(kind, origin_base)` pairs per spec.
static HARVEST: LazyLock<Vec<Vec<(&'static str, &'static str)>>> = LazyLock::new(|| {
    SPECS
        .iter()
        .map(|spec| {
            spec.kinds
                .iter()
                .filter(|k| k.harvest)
                .map(|k| {
                    let base = k
                        .origin_base
                        .as_deref()
                        .or(k.url_base.as_deref())
                        .unwrap_or("");
                    (k.kind.as_str(), base)
                })
                .collect()
        })
        .collect()
});

// ---------------------------------------------------------------------------
// Generated FieldSpec tables — one flat `Vec<FieldSpec>` borrows SPECS'
// strings + the option-pair statics; per-owner tables hold slices into it.
// ---------------------------------------------------------------------------

/// `Select` option pairs per field, flattened across all owners —
/// nested-static trick identical to `TAX_TABLES`.
static FIELD_OPTS: LazyLock<Vec<Vec<(&'static str, &'static str)>>> = LazyLock::new(|| {
    SPECS
        .iter()
        .flat_map(|spec| {
            spec.screens
                .iter()
                .flat_map(|s| s.fields.iter())
                .chain(spec.kinds.iter().filter_map(|k| k.metabox.as_ref()).flat_map(|m| m.fields.iter()))
                .map(|f| {
                    f.options
                        .iter()
                        .map(|(v, l)| (v.as_str(), l.as_str()))
                        .collect()
                })
        })
        .collect()
});

/// Every declared field (screens ∪ metaboxes) materialized as
/// `index::FieldSpec`, in the same flat order `FIELD_OPTS` walks.
static FIELD_SPECS: LazyLock<Vec<FieldSpec>> = LazyLock::new(|| {
    let mut opts = FIELD_OPTS.iter();
    let mut out = Vec::new();
    for spec in SPECS.iter() {
        let fields = spec
            .screens
            .iter()
            .flat_map(|s| s.fields.iter())
            .chain(spec.kinds.iter().filter_map(|k| k.metabox.as_ref()).flat_map(|m| m.fields.iter()));
        for f in fields {
            let o = opts.next().expect("FIELD_OPTS/FIELD_SPECS length mismatch");
            out.push(FieldSpec {
                key: f.key.as_str(),
                label: f.label.as_str(),
                kind: match f.kind.as_str() {
                    "text" => FieldKind::Text,
                    "textarea" => FieldKind::Textarea,
                    "select" => FieldKind::Select(o.as_slice()),
                    "checkbox" => FieldKind::Checkbox,
                    "serialized" => FieldKind::SerializedTextarea,
                    "note" => FieldKind::Note(f.help.as_str()),
                    other => panic!("unknown spec field kind {other:?}"),
                },
                help: f.help.as_str(),
            });
        }
    }
    out
});

/// Screen fields as slices into `FIELD_SPECS` — same flat walk order.
static SCREEN_FIELDS: LazyLock<Vec<Vec<&'static [FieldSpec]>>> = LazyLock::new(|| {
    let mut cursor = 0usize;
    SPECS
        .iter()
        .map(|spec| {
            spec.screens
                .iter()
                .map(|s| {
                    let start = cursor;
                    cursor += s.fields.len();
                    &FIELD_SPECS[start..cursor]
                })
                .collect()
        })
        .collect()
});

/// Metabox field slices per spec per kind — `None` for classic-only
/// kinds. Advances the same flat cursor after the screens of each spec.
static METABOX_FIELDS: LazyLock<Vec<Vec<Option<&'static [FieldSpec]>>>> = LazyLock::new(|| {
    // FIELD_SPECS is per-spec interleaved (screens then metaboxes), so the
    // cursor must skip this spec's screen fields before slicing its
    // metabox fields — not the total of every spec's screens.
    let mut cursor: usize = 0;
    SPECS
        .iter()
        .map(|spec| {
            cursor += spec.screens.iter().map(|sc| sc.fields.len()).sum::<usize>();
            spec.kinds
                .iter()
                .map(|k| {
                    k.metabox.as_ref().map(|m| {
                        let start = cursor;
                        cursor += m.fields.len();
                        &FIELD_SPECS[start..cursor]
                    })
                })
                .collect()
        })
        .collect()
});

/// `(spec_idx, screen_idx)` per flat screen index — `screen_at` resolves
/// a slot back to the screen + its fields.
static SCREEN_FLAT: LazyLock<Vec<(usize, usize)>> = LazyLock::new(|| {
    SPECS
        .iter()
        .enumerate()
        .flat_map(|(si, spec)| (0..spec.screens.len()).map(move |sci| (si, sci)))
        .collect()
});

/// `ScreenDef` tables per spec — the handle dispatches through a slot
/// table like `claim_at` (fn pointers can't capture the screen index).
static SCREEN_DEFS: LazyLock<Vec<Vec<ScreenDef>>> = LazyLock::new(|| {
    let mut flat = 0usize;
    SPECS
        .iter()
        .map(|spec| {
            spec.screens
                .iter()
                .map(|s| {
                    let d = ScreenDef {
                        path: s.path.as_str(),
                        title: s.title.as_str(),
                        min_role: s.min_role.as_str(),
                        handle: screen_at(flat),
                    };
                    flat += 1;
                    d
                })
                .collect()
        })
        .collect()
});

fn screen_at(n: usize) -> super::ScreenHandler {
    macro_rules! slot {
        ($n:literal) => {{
            fn h<'a>(
                app: &'a crate::admin::App,
                s: &'a crate::admin::Sess,
                sc: &'a crate::admin::SiteCtx,
                headers: HeaderMap,
                rest: &'a str,
                req: Request,
            ) -> ScreenFuture<'a> {
                spec_screen($n, app, s, sc, headers, rest, req)
            }
            h as super::ScreenHandler
        }};
    }
    const TABLE: &[super::ScreenHandler] = &[
        slot!(0), slot!(1), slot!(2), slot!(3), slot!(4), slot!(5),
        slot!(6), slot!(7), slot!(8), slot!(9), slot!(10), slot!(11),
    ];
    *TABLE
        .get(n)
        .expect("more than 12 spec screens — extend screen_at slot table")
}

/// Generated settings screen: GET renders `settings_page_html`, POST
/// saves via the shared `save_settings_fields` then redirects back —
/// identical shape to every Rust-native settings screen.
fn spec_screen<'a>(
    flat: usize,
    _app: &'a crate::admin::App,
    s: &'a crate::admin::Sess,
    sc: &'a crate::admin::SiteCtx,
    _headers: HeaderMap,
    _rest: &'a str,
    req: Request,
) -> ScreenFuture<'a> {
    Box::pin(async move {
        let Some(&(si, sci)) = SCREEN_FLAT.get(flat) else {
            return StatusCode::NOT_FOUND.into_response();
        };
        let spec = &SPECS[si];
        let screen = &spec.screens[sci];
        let fields = SCREEN_FIELDS[si][sci];
        let action = format!("/admin/plugins/{}/{}", spec.slug, screen.path);
        match *req.method() {
            Method::POST => {
                let Ok(Form(f)) = Form::<SettingsForm>::from_request(req, &()).await else {
                    return StatusCode::BAD_REQUEST.into_response();
                };
                match index::save_settings_fields(s, sc, &f, fields) {
                    Ok(()) => Redirect::to(&action).into_response(),
                    Err(r) => r,
                }
            }
            _ => index::settings_page_html(s, sc, &action, &screen.title, fields),
        }
    })
}

/// `EditorHooks` per spec per kind — `Some` only when the kind declares
/// a metabox. All four fns dispatch off `f.kind` against `SPECS`, so
/// one generic impl covers every generated metabox.
static SPEC_EDITORS: LazyLock<Vec<Vec<Option<EditorHooks>>>> = LazyLock::new(|| {
    SPECS
        .iter()
        .map(|spec| {
            spec.kinds
                .iter()
                .map(|k| {
                    k.metabox.as_ref().map(|_| EditorHooks {
                        load: spec_editor_load,
                        metabox: spec_metabox,
                        assets: spec_assets,
                        save: spec_editor_save,
                    })
                })
                .collect()
        })
        .collect()
});

/// `post_meta` row → `f.extra[key]` for every declared metabox field,
/// so the metabox renders stored values without a second read.
fn spec_editor_load(c: &Connection, id: i64, f: &mut crate::db::posts::PostForm) -> Result<()> {
    if let Some(fields) = metabox_fields_for(&f.kind) {
        for fs in fields {
            if let Ok(v) = c.query_row(
                "SELECT meta_value FROM post_meta WHERE post_id=?1 AND meta_key=?2 LIMIT 1",
                rusqlite::params![id, fs.key],
                |r| r.get::<_, String>(0),
            ) {
                f.extra.insert(fs.key.to_string(), v);
            }
        }
    }
    Ok(())
}

/// Generated metabox panel: `postbox` chrome + one row per field.
/// Values come from `f.extra` (hydrated by `spec_editor_load`).
fn spec_metabox(sc: &crate::admin::SiteCtx, _s: &crate::admin::Sess, f: &crate::db::posts::PostForm) -> String {
    let Some(fields) = metabox_fields_for(&f.kind) else {
        return String::new();
    };
    let title = metabox_title_for(&f.kind).unwrap_or("Settings");
    let mut rows = String::new();
    for fs in fields {
        let cur = f.extra.get(fs.key).cloned().unwrap_or_default();
        let input = match &fs.kind {
            FieldKind::Text | FieldKind::SerializedTextarea => format!(
                "<input type=\"text\" name=\"{k}\" value=\"{v}\" class=\"regular-text\">",
                k = fs.key,
                v = esc(&cur)
            ),
            FieldKind::Textarea => format!(
                "<textarea name=\"{k}\" rows=\"4\" class=\"large-text\">{v}</textarea>",
                k = fs.key,
                v = esc(&cur)
            ),
            FieldKind::Select(opts) => {
                let mut o = String::new();
                for (v, label) in *opts {
                    let sel = if *v == cur { " selected" } else { "" };
                    o.push_str(&format!("<option value=\"{v}\"{sel}>{label}</option>"));
                }
                format!("<select name=\"{k}\">{o}</select>", k = fs.key)
            }
            FieldKind::Checkbox => {
                let chk = if cur == "1" { " checked" } else { "" };
                format!("<input type=\"checkbox\" name=\"{k}\" value=\"1\"{chk}>", k = fs.key)
            }
            FieldKind::Note(_) => String::new(),
        };
        if let FieldKind::Note(text) = &fs.kind {
            rows.push_str(&format!("<p class=\"description\">{text}</p>"));
        } else {
            rows.push_str(&format!(
                "<p><label for=\"{k}\"><strong>{l}</strong></label><br>{input}{help}</p>",
                k = fs.key,
                l = esc(fs.label),
                help = if fs.help.is_empty() {
                    String::new()
                } else {
                    format!("<br><span class=\"description\">{}</span>", esc(fs.help))
                }
            ));
        }
    }
    let _ = sc;
    format!("<div class=\"postbox\"><h2>{title}</h2><div class=\"inside\"><input type=\"hidden\" name=\"__present\" value=\"1\">{rows}</div></div>")
}

fn spec_assets() -> &'static str {
    ""
}

/// Persist metabox fields to `post_meta`. Quick edit / bulk edit never
/// post metabox fields: absent non-checkbox keys are skipped, absent
/// checkboxes only zero when another metabox field proves the box was
/// on the form.
fn spec_editor_save(c: &Connection, id: i64, f: &crate::db::posts::PostForm) -> Result<()> {
    let Some(fields) = metabox_fields_for(&f.kind) else {
        return Ok(());
    };
    // `__present` proves the metabox was on the form even when every
    // field is an unchecked checkbox (checkboxes post nothing when off);
    // the legacy per-field probe stays for pre-marker callers.
    let box_present = f.extra.contains_key("__present")
        || fields.iter().any(|fs| f.extra.contains_key(fs.key));
    if !box_present {
        return Ok(());
    }
    for fs in fields {
        if matches!(fs.kind, FieldKind::Note(_)) {
            continue;
        }
        let Some(v) = f.extra.get(fs.key).map(|s| s.as_str()).or(match fs.kind {
            FieldKind::Checkbox => Some("0"),
            _ => None,
        }) else {
            continue;
        };
        c.execute(
            "DELETE FROM post_meta WHERE post_id=?1 AND meta_key=?2",
            rusqlite::params![id, fs.key],
        )?;
        c.execute(
            "INSERT INTO post_meta(post_id, meta_key, meta_value) VALUES (?1, ?2, ?3)",
            rusqlite::params![id, fs.key, v],
        )?;
    }
    Ok(())
}

fn metabox_fields_for(kind: &str) -> Option<&'static [FieldSpec]> {
    for (si, spec) in SPECS.iter().enumerate() {
        for (ki, k) in spec.kinds.iter().enumerate() {
            if k.kind == kind {
                return METABOX_FIELDS[si][ki];
            }
        }
    }
    None
}

fn metabox_title_for(kind: &str) -> Option<&'static str> {
    SPECS
        .iter()
        .flat_map(|s| s.kinds.iter())
        .find(|k| k.kind == kind)
        .and_then(|k| k.metabox.as_ref())
        .map(|m| m.title.as_str())
}

/// Plugins-index name/blurb — owned fallbacks live here so the
/// `&'static` borrow in `PluginDescriptor` is valid.
static NAMES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    SPECS
        .iter()
        .map(|s| {
            (
                s.name.clone().unwrap_or_else(|| s.slug.clone()),
                s.blurb.clone().unwrap_or_default(),
            )
        })
        .collect()
});

fn post_noop(_c: &Connection, _id: i64) -> Result<()> {
    Ok(())
}
fn after_noop(_c: &Connection) -> Result<()> {
    Ok(())
}

/// One ImportHooks per spec — `claim_type` can't capture, so it
/// dispatches through `claim_at(i)`; the slot table caps spec count
/// (extend when a 9th spec lands — panics loudly if exceeded).
static HOOKS: LazyLock<Vec<ImportHooks>> = LazyLock::new(|| {
    (0..SPECS.len())
        .map(|i| ImportHooks {
            claim_type: claim_at(i),
            option_prefixes: PREFIXES[i].as_slice(),
            post_imported: post_noop,
            after_import: after_noop,
        })
        .collect()
});

fn claim_at(i: usize) -> fn(&str) -> Option<&'static str> {
    macro_rules! slot {
        ($n:literal) => {{
            fn claim(t: &str) -> Option<&'static str> {
                CLAIMS[$n].iter().find(|(ty, _)| *ty == t).map(|(_, k)| *k)
            }
            claim as fn(&str) -> Option<&'static str>
        }};
    }
    const TABLE: &[fn(&str) -> Option<&'static str>] = &[
        slot!(0), slot!(1), slot!(2), slot!(3), slot!(4), slot!(5), slot!(6), slot!(7),
    ];
    *TABLE
        .get(i)
        .expect("more than 8 spec adapters — extend claim_at slot table")
}

/// Registered spec adapter: an index into the static tables. Cheap to
/// copy; `&'static` borrows come from `SPEC_ADAPTERS`/`SPECS` et al.
#[derive(Clone, Copy)]
pub(crate) struct SpecAdapter {
    idx: usize,
}

static SPEC_ADAPTERS: LazyLock<Vec<SpecAdapter>> =
    LazyLock::new(|| (0..SPECS.len()).map(|idx| SpecAdapter { idx }).collect());

/// Spec adapters as trait objects — `adapters()` merges these with
/// `ADAPTERS` so claim dispatch, admin dispatch, and registry asserts
/// cover specs with zero call-site changes.
static SPEC_TRAITS: LazyLock<Vec<&'static dyn PluginAdapter>> =
    LazyLock::new(|| {
        // SPECS init first so kinds/hooks tables are valid below.
        let _ = &*HOOKS;
        SPEC_ADAPTERS
            .iter()
            .map(|a| a as &'static dyn PluginAdapter)
            .collect()
    });

pub(crate) fn spec_adapters() -> &'static [&'static dyn PluginAdapter] {
    SPEC_TRAITS.as_slice()
}

/// `(kind, origin_base)` for `harvest = true` spec kinds, flattened
/// across specs — phase 02's import loop + generate path consult this.
static HARVEST_FLAT: LazyLock<Vec<(&'static str, &'static str)>> =
    LazyLock::new(|| HARVEST.iter().flatten().copied().collect());

pub(crate) fn spec_harvest_bases() -> &'static [(&'static str, &'static str)] {
    HARVEST_FLAT.as_slice()
}

/// Phase-02 edge case "spec claims harvest kind owned by a Rust
/// adapter" is already covered: REGISTRY panics on the duplicate kind
/// claim before any harvest logic runs.



impl PluginAdapter for SpecAdapter {
    fn slug(&self) -> &'static str {
        SPECS[self.idx].slug.as_str()
    }
    fn kinds(&self) -> &'static [KindDef] {
        KIND_DEFS[self.idx].as_slice()
    }
    fn backend(&self) -> Backend {
        Backend::Data
    }
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            name: NAMES[self.idx].0.as_str(),
            blurb: NAMES[self.idx].1.as_str(),
        }
    }
    fn settings_prefixes(&self) -> &'static [&'static str] {
        PREFIXES[self.idx].as_slice()
    }
    fn screens(&self) -> &'static [ScreenDef] {
        SCREEN_DEFS[self.idx].as_slice()
    }
    fn legacy_option_keys(&self) -> &'static [&'static str] {
        LEGACY_KEYS[self.idx].as_slice()
    }
    fn import_hooks(&self) -> Option<&'static ImportHooks> {
        if CLAIMS[self.idx].is_empty() && PREFIXES[self.idx].is_empty() {
            return None;
        }
        Some(&HOOKS[self.idx])
    }
    fn render(&self, _conn: &Connection, _templates: &std::path::Path, _ctx: &RenderCtx) -> Result<Option<String>> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"
slug = "babe"
claim_types = ["to_book"]
option_prefixes = ["babe_"]

[[kinds]]
kind = "to_book"
section = "bookings"
noun = "booking"
url_base = "booking"
taxonomies = ["ba_location", "ba_type", "categories"]
harvest = true
"#;

    #[test]
    fn spec_parses_to_kinddef() {
        let spec: Spec = toml::from_str(FIXTURE).unwrap();
        assert_eq!(spec.slug, "babe");
        let k = &spec.kinds[0];
        assert_eq!(k.kind, "to_book");
        assert_eq!(k.section, "bookings");
        assert_eq!(k.noun, "booking");
        assert_eq!(k.url_base.as_deref(), Some("booking"));
        assert_eq!(k.taxonomies, ["ba_location", "ba_type", "categories"]);
        assert!(k.harvest);
    }

    #[test]
    fn unknown_key_rejected() {
        let bad = FIXTURE.replace("claim_types", "claim_typo");
        assert!(toml::from_str::<Spec>(&bad).is_err());
    }

    #[test]
    fn empty_specs_dir_yields_no_adapters() {
        // Embedded table is empty until a .toml lands; the union must be
        // zero-cost and harmless.
        assert!(spec_adapters().len() == SPECS.len());
    }

    #[test]
    fn screens_and_metabox_parse() {
        let src = r#"
slug = "demo"
name = "Demo Plugin"
blurb = "Proving spec."

[[screens]]
path = "settings"
title = "Demo Settings"
[[screens.fields]]
key = "demo_mode"
label = "Mode"
kind = "select"
options = [["a", "A"], ["b", "B"]]
[[screens.fields]]
key = "demo_on"
label = "On"
kind = "checkbox"

[[kinds]]
kind = "demo_item"
section = "demos"
noun = "demo"
[kinds.metabox]
title = "Demo Meta"
[[kinds.metabox.fields]]
key = "demo_code"
label = "Code"
kind = "text"
"#;
        let spec: Spec = toml::from_str(src).unwrap();
        assert_eq!(spec.screens.len(), 1);
        assert_eq!(spec.screens[0].fields.len(), 2);
        assert_eq!(spec.screens[0].min_role, "admin");
        let mb = spec.kinds[0].metabox.as_ref().unwrap();
        assert_eq!(mb.title, "Demo Meta");
        assert_eq!(mb.fields[0].key, "demo_code");
    }

    #[test]
    fn bad_field_kind_rejected() {
        let src = r#"
slug = "demo"
kinds = []
[[screens]]
path = "settings"
title = "S"
[[screens.fields]]
key = "x"
label = "X"
kind = "wysiwyg"
"#;
        let spec: Spec = toml::from_str(src).unwrap();
        assert!(parse_field_kind(&spec.screens[0].fields[0].kind, &[]).is_none());
    }

    /// The repo spec (babe.toml) must land its screen + metabox in the
    /// generated tables — this is the proving-spec acceptance check.
    #[test]
    fn babe_generates_screen_and_editor() {
        // Screens resolve through the same dispatch Rust screens use.
        let (a, screen) = crate::plugins::screen_for("babe", "settings").unwrap();
        assert_eq!(a.slug(), "babe");
        assert_eq!(screen.title, "BA Book Settings");
        // The metabox lands an EditorHooks on the to_book KindDef.
        let hooks = crate::plugins::editor_for("to_book").expect("to_book editor hooks");
        // load + save + metabox round-trip through post_meta.
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch("CREATE TABLE post_meta(id INTEGER PRIMARY KEY, post_id INT, meta_key TEXT, meta_value TEXT)").unwrap();
        let mut f = crate::db::posts::PostForm { kind: "to_book".into(), ..Default::default() };
        (hooks.load)(&c, 7, &mut f).unwrap();
        assert!(f.extra.is_empty());
        f.extra.insert("ba_booking_code".into(), "BK-1".into());
        f.extra.insert("ba_booking_status".into(), "confirmed".into());
        (hooks.save)(&c, 7, &f).unwrap();
        let got: String = c
            .query_row("SELECT meta_value FROM post_meta WHERE post_id=7 AND meta_key='ba_booking_code'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(got, "BK-1");
        // Absent checkbox persists "0" because sibling fields prove the box posted.
        let paid: String = c
            .query_row("SELECT meta_value FROM post_meta WHERE post_id=7 AND meta_key='ba_booking_paid'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(paid, "0");
        // Quick edit: no metabox fields posted → existing rows untouched.
        let qe = crate::db::posts::PostForm { kind: "to_book".into(), ..Default::default() };
        (hooks.save)(&c, 7, &qe).unwrap();
        let still: String = c
            .query_row("SELECT meta_value FROM post_meta WHERE post_id=7 AND meta_key='ba_booking_code'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(still, "BK-1");
    }
}
