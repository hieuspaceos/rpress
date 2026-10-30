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

use super::{Backend, ImportHooks, KindDef, PluginAdapter, RenderCtx};
use crate::admin::posts;
use crate::db;
use anyhow::Result;
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
    pub(crate) kinds: Vec<KindSpec>,
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
}

fn default_menu_icon() -> String {
    "dashicons-tickets-alt".to_string()
}

const HEAD_SPEC: &str = r#"<tr><td class="manage-column column-cb check-column"></td><th scope="col" class="manage-column column-title column-primary">Title</th><th scope="col">Categories</th><th scope="col">FR / ES</th><th scope="col">Comments</th><th scope="col" class="rm-seo-col">SEO Details</th></tr>"#;

fn spec_row_cols(r: &db::AdminListRow, kind: &str) -> String {
    format!(
        "<td class=\"categories\">{}</td><td class=\"pll\">{}</td>",
        posts::dash(&r.categories),
        posts::pll_cell(r, kind),
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
            spec
        })
        .collect()
});

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
                    editor: None,
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
}
