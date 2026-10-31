//! TOML theme specs — `themes/specs/*.toml` embedded by build.rs and
//! parsed once into `ThemeDef`-shaped data. Declarative subset of the
//! Rust theme contract: URL shapes (tag base, taxonomy→lang), class
//! prefixes, admin labels. A theme with code-level quirks (goodlayers
//! metaboxes, seed widgets) keeps a Rust adapter — the escape hatch wins
//! on stylesheet collision (panic at registry init, same as plugin specs).

use super::ThemeDef;
use serde::Deserialize;
use std::sync::LazyLock;

include!(concat!(env!("OUT_DIR"), "/theme_specs.rs"));

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ThemeSpec {
    /// Stylesheet names this theme claims (`traveltour*`, `tevily*`).
    /// `*` is a trailing wildcard only — anything fancier needs Rust.
    stylesheet_patterns: Vec<String>,
    template_dir: String,
    #[serde(default)]
    body_suffix: String,
    /// Ignored — the generic `default_page_title` covers data themes;
    /// consumed below so `deny_unknown_fields` keeps the TOML key valid.
    #[serde(default)]
    page_title_wrap: Option<String>,
    #[serde(default)]
    preferred_locations: Vec<String>,
    #[serde(default)]
    taxonomy_map_key: String,
    #[serde(default)]
    widget_class: String,
    #[serde(default)]
    sidebar_wrap_class: String,
    #[serde(default)]
    body_class: String,
    #[serde(default)]
    settings_key_prefix: String,
    #[serde(default)]
    tag_base: Option<String>,
    #[serde(default)]
    tax_lang: Vec<TaxLang>,
    /// Primary↔twin taxonomy pairs: [["style","estilo"],…] — feeds
    /// `localized_taxonomy`; twins' lang resolves via `tax_lang`.
    #[serde(default)]
    tax_pairs: Vec<(String, String)>,
    /// Menu markup variant: "wp" (default) or "caret"/"gva"/"elementor"
    /// (tevily-shaped <span class="menu-title"> + caret).
    #[serde(default)]
    menu_renderer: String,
    #[serde(default)]
    tax_labels: Vec<TaxLabel>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TaxLang {
    taxonomy: String,
    lang: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TaxLabel {
    taxonomy: String,
    plural: String,
    singular: String,
}

/// Parsed specs with their owned strings leaked to 'static (registry is
/// built once at first resolve — same pattern as `SPECS`).
static THEMES: LazyLock<Vec<(Vec<String>, &'static ThemeDef)>> = LazyLock::new(|| {
    THEME_SPEC_SRCS
        .iter()
        .map(|(slug, src)| {
            let spec: ThemeSpec = toml::from_str(src)
                .unwrap_or_else(|e| panic!("themes/specs/{slug}.toml: {e}"));
            // Leak owned Strings into &'static — ThemeDef fields are
            // 'static; registry data never freed.
            let leak = |s: &String| -> &'static str {
                Box::leak(s.clone().into_boxed_str())
            };
            let patterns: Vec<String> = spec.stylesheet_patterns.clone();
            let def: &'static ThemeDef = Box::leak(Box::new(ThemeDef {
                matches_stylesheet: |_| false, // unused — patterns decide below
                template_dir: leak(&spec.template_dir),
                body_suffix: leak(&spec.body_suffix),
                page_title_wrap: super::default_page_title,
                preferred_locations: {
                    let v: Vec<&'static str> =
                        spec.preferred_locations.iter().map(leak).collect();
                    Box::leak(v.into_boxed_slice())
                },
                taxonomy_map_key: leak(&spec.taxonomy_map_key),
                taxonomy_map_json: "",
                tag_base: spec.tag_base.as_deref().map(|s| Box::leak(s.to_string().into_boxed_str()) as &'static str).unwrap_or(""),
                widget_class: if spec.widget_class.is_empty() {
                    "widget"
                } else {
                    leak(&spec.widget_class)
                },
                sidebar_wrap_class: if spec.sidebar_wrap_class.is_empty() {
                    "content-container sidebar-wrap"
                } else {
                    leak(&spec.sidebar_wrap_class)
                },
                body_class: leak(&spec.body_class),
                settings_key_prefix: leak(&spec.settings_key_prefix),
                tax_lang: {
                    let v: Vec<(&'static str, &'static str)> = spec
                        .tax_lang
                        .iter()
                        .map(|t| (leak(&t.taxonomy), leak(&t.lang)))
                        .collect();
                    Box::leak(v.into_boxed_slice())
                },
                tax_pairs: {
                    let v: Vec<(&'static str, &'static str)> = spec
                        .tax_pairs
                        .iter()
                        .map(|t| (leak(&t.0), leak(&t.1)))
                        .collect();
                    Box::leak(v.into_boxed_slice())
                },
                tax_labels: {
                    let v: Vec<(&'static str, &'static str, &'static str)> = spec
                        .tax_labels
                        .iter()
                        .map(|t| (leak(&t.taxonomy), leak(&t.plural), leak(&t.singular)))
                        .collect();
                    Box::leak(v.into_boxed_slice())
                },
                css_link_id: "",
                js_extra_id: "",
                wrap_page_sidebar: super::default_page_sidebar,
                mega_class: "mega-menu",
                normal_menu_class: "normal-menu",
                menu_renderer: match spec.menu_renderer.as_str() {
                    "caret" | "gva" | "elementor" => super::MenuRenderer::Caret,
                    _ => super::MenuRenderer::Wp,
                },
            }));
            let _ = &spec.page_title_wrap;
            (patterns, def)
        })
        .collect()
});

fn pattern_match(pat: &str, stylesheet: &str) -> bool {
    if let Some(p) = pat.strip_suffix('*') {
        stylesheet.starts_with(p)
    } else {
        pat == stylesheet
    }
}

/// TOML theme for `stylesheet`, if any spec claims it. Rust adapters are
/// checked first by `theme_for_site` — this is the second tier.
pub(crate) fn theme_for_stylesheet(stylesheet: &str) -> Option<&'static ThemeDef> {
    THEMES
        .iter()
        .find(|(pats, _)| pats.iter().any(|p| pattern_match(p, stylesheet)))
        .map(|(_, d)| *d)
}

/// Collision check — a TOML theme must not claim a stylesheet a Rust
/// adapter already owns. Called once at registry init from
/// `assert_theme_unique` (the plugin-spec `claim_types` equivalent).
pub(crate) fn assert_no_rust_collision() {
    for (pats, _) in THEMES.iter() {
        for adapter in super::adapters() {
            if let Some(t) = adapter.theme() {
                for probe in pats {
                    let probe = probe.trim_end_matches('*');
                    if (t.matches_stylesheet)(probe) {
                        panic!("theme spec claims stylesheet '{probe}' already owned by adapter {}", adapter.slug());
                    }
                }
            }
        }
    }
}
