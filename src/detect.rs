//! `rpress detect` — inspect a WP dump (+ optional pulled wp-content) and
//! report which plugins/themes/post_types a site maps to which rpress
//! backend tier, with concrete next steps (env line, spec draft, adapter
//! work). rpress has no capability negotiation at runtime: adapters are
//! globally registered and sites opt in via `PLUGINS_PHP`. This command
//! is the discovery layer a human/agent runs BEFORE onboarding.
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Markers in plugin PHP source that hint at a builder/runtime plugin —
/// deeper than "has admin screens", used to guess tier-3 work.
const BUILDER_MARKERS: &[(&str, &str)] = &[
    ("Widget_Base", "elementor-style widgets"),
    ("page_builder_element", "goodlayers-style builder elements"),
    ("WPBakeryShortCode", "wpbakery/vc builder"),
    ("et_pb_module", "divi builder modules"),
];

/// `(dir_slug, plugin_file)` per active plugin — `active_plugins` is a
/// serialized PHP array of `dir/file.php` strings.
fn active_plugins(dump: &Path) -> Vec<(String, String)> {
    use crate::admin::bridge::playground::option_from_dump;
    let Some(raw) = option_from_dump(Some(dump), "active_plugins") else {
        eprintln!("detect: option_from_dump(active_plugins) → None");
        return Vec::new();
    };
    match crate::php_serialize::unserialize(raw.as_bytes()) {
        // php_serialize returns Object for assoc arrays and Array for
        // sequential ones — active_plugins is sequential but its int
        // keys can surface either shape; take values uniformly.
        Ok(serde_json::Value::Array(items)) => collect(items.iter().filter_map(|i| i.as_str()).map(str::to_string)),
        Ok(serde_json::Value::Object(m)) => collect(m.into_values().filter_map(|i| i.as_str().map(str::to_string))),
        Ok(_) => {
            eprintln!("detect: active_plugins parsed but not an array");
            Vec::new()
        }
        Err(e) => {
            eprintln!("detect: unserialize(active_plugins) failed: {e}");
            Vec::new()
        }
    }
}

fn collect(files: impl Iterator<Item = String>) -> Vec<(String, String)> {
    files
        .map(|f| {
            let dir = f.split('/').next().unwrap_or(&f).to_string();
            (dir, f)
        })
        .collect()
}

/// Distinct wp post_types with counts.
fn post_types(dump: &Path) -> BTreeMap<String, u64> {
    let prefix = crate::sqldump::detect_prefix(dump).unwrap_or_else(|_| "wp_".into());
    let table = format!("{prefix}posts");
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    let _ = crate::sqldump::rows(dump, &[table.as_str()], |_, v| {
        // wp_posts column 20 = post_type (import.rs row mapping).
        if let Some(t) = v.get(20).and_then(|x| x.to_string_lossy()) {
            *counts.entry(t).or_default() += 1;
        }
        Ok(())
    });
    counts
}

/// Scan a plugin dir for builder markers (capped walk — vendor/tests
/// skipped so test fixtures can't produce false positives).
fn scan_markers(dir: &Path) -> BTreeSet<&'static str> {
    let mut found = BTreeSet::new();
    if !dir.is_dir() {
        return found;
    }
    let mut stack = vec![dir.to_path_buf()];
    let mut files = 0usize;
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                let n = e.file_name();
                if n != "vendor" && n != "vendor_prefixed" && n != "tests" && n != "node_modules" {
                    stack.push(p);
                }
            } else if p.extension().map(|x| x == "php").unwrap_or(false) {
                files += 1;
                if files > 4000 {
                    return found;
                }
                let Ok(s) = std::fs::read_to_string(&p) else { continue };
                for (marker, label) in BUILDER_MARKERS {
                    if s.contains(marker) {
                        found.insert(*label);
                    }
                }
            }
        }
    }
    found
}

/// The wp post_types already claimed: adapter claim_type is a fn, so
/// probe every observed type rather than inverting the fns.
fn claimed<'a>(types: impl Iterator<Item = &'a String>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for t in types {
        for a in crate::plugins::adapters() {
            if a.import_hooks().and_then(|h| (h.claim_type)(t)).is_some() {
                out.insert(t.clone());
            }
        }
    }
    // Core claims — mirror import.rs kind_of's own match arm.
    for t in ["post", "page", "site-review", "attachment", "nav_menu_item", "revision",
              "wpcf7_contact_form", "custom_css", "elementor_library", "wp_template",
              "wp_theme", "oembed_cache", "wp_block", "wp_navigation", "user_request",
              "customize_changeset"] {
        out.insert(t.to_string());
    }
    out
}

pub fn run(dump: &Path, www: Option<&Path>) -> Result<()> {
    if !dump.is_file() {
        anyhow::bail!("dump not found: {}", dump.display());
    }
    use crate::admin::bridge::playground::option_from_dump;
    let plugins_dir = www.map(|w| w.join("wp-content").join("plugins"));

    println!("== theme ==");
    match option_from_dump(Some(dump), "stylesheet") {
        Some(ss) => {
            // Child theme: render uses stylesheet; bridge needs `template`
            // (the parent) — report both, matching the wp-config render.
            let parent = option_from_dump(Some(dump), "template").unwrap_or_default();
            // Same chain as runtime theme_for_site: Rust adapter →
            // TOML spec → default.
            match crate::plugins::theme_for_stylesheet(&ss)
                .or_else(|| crate::plugins::themes::theme_for_stylesheet(&ss))
            {
                Some(_) => println!("  {ss} (parent: {parent}) → ThemeDef matched — chrome/import covered"),
                None => println!("  {ss} (parent: {parent}) → NO ThemeDef — menus/chrome fall back to DEFAULT_THEME; consider a theme spec"),
            }
        }
        None => println!("  no stylesheet option in dump"),
    }

    println!("\n== active plugins ==");
    let mut php_candidates: Vec<String> = Vec::new();
    let active = active_plugins(dump);
    if active.is_empty() {
        println!("  none found (active_plugins missing/unparseable)");
    }
    for (dir_slug, file) in &active {
        let known = crate::plugins::adapters()
            .iter()
            .find(|a| a.slug() == dir_slug || a.wp_plugin_file() == file);
        let markers = plugins_dir
            .as_ref()
            .map(|pd| scan_markers(&pd.join(dir_slug)))
            .unwrap_or_default();
        match known {
            Some(a) => {
                let backend = format!("{:?}", a.backend());
                println!("  {file} → adapter '{}' ({backend})", a.slug());
                if backend == "PhpBridge" {
                    php_candidates.push(a.slug().to_string());
                }
            }
            None if !markers.is_empty() => {
                println!("  {file} → NO adapter — markers: {}", markers.iter().cloned().collect::<Vec<_>>().join(", "));
                println!("      tier-3 candidate: needs a PhpBridge adapter (query-forward + plugin-activation filter + write_specs)");
                php_candidates.push(format!("<{dir_slug}: adapter needed>"));
            }
            None => {
                println!("  {file} → NO adapter — data-only for now (post_meta/options import; admin screens absent)");
            }
        }
    }

    let types = post_types(dump);
    let claimed_set = claimed(types.keys());
    println!("\n== post types ==");
    let mut spec_hints: Vec<(String, Option<String>)> = Vec::new();
    let core_skip = |t: &str| matches!(t, "attachment" | "revision" | "nav_menu_item"
        | "customize_changeset" | "wp_font_face" | "wp_font_family" | "wp_global_styles");
    for (t, n) in &types {
        if core_skip(t) {
            continue;
        }
        if claimed_set.contains(t) {
            println!("  {t} ({n}) → claimed");
        } else {
            println!("  {t} ({n}) → UNCLAIMED — falls to generic post/page or is skipped");
            // Attribute an unclaimed type to a plugin by name prefix —
            // a guess, labeled as one.
            let guess = active
                .iter()
                .map(|(d, _)| d.clone())
                .find(|d| {
                    let d = d.replace('-', "_");
                    t.contains(&d) || t.starts_with(d.split('_').next().unwrap_or("~"))
                });
            spec_hints.push((t.clone(), guess));
        }
    }

    println!("\n== suggested next steps ==");
    if !php_candidates.is_empty() {
        println!("  PLUGINS_PHP={}", php_candidates.join(","));
    }
    for (t, guess) in &spec_hints {
        let owner = guess.as_deref().unwrap_or("<plugin-slug>");
        println!("  unclaimed type {t} → draft plugins/specs/{owner}.toml: claim_types += [\"{t}\"]");
    }
    println!("\nNOTE: claims are structural guesses. `Backend` choice (Rust port vs Data vs PhpBridge) stays a human decision — review claim types, option prefixes, and whether admin screens matter for this site.");
    Ok(())
}
