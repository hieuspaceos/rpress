//! report which plugins/themes/post_types a site maps to which rpress
//! backend tier, with concrete next steps (env line, spec draft, adapter
//! work). rpress has no capability negotiation at runtime: adapters are
//! globally registered and sites opt in via `PLUGINS_PHP`. This command
//! is the discovery layer a human/agent runs BEFORE onboarding.
//!
//! Coverage (deep → shallow):
//!   1. theme stylesheet → ThemeDef (Rust → TOML spec → default)
//!   2. active plugins → adapter match + builder-marker scan
//!   3. wp post_types → claimed, else attributed to the plugin that
//!      registers them (register_post_type source scan, not a name guess)
//!   4. shortcodes actually used in post_content
//!   5. wp_options prefixes → covered by adapter option_prefixes or not
//!   6. plugin/theme dirs present on disk but inactive
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Markers in plugin PHP source that hint at a builder/runtime plugin —
/// deeper than "has admin screens", used to guess tier-3 work.
const BUILDER_MARKERS: &[(&str, &str)] = &[
    ("Widget_Base", "elementor widgets"),
    ("page_builder_element", "goodlayers builder elements"),
    ("WPBakeryShortCode", "wpbakery/vc builder"),
    ("et_pb_module", "divi builder modules"),
];

/// Taxonomies/internal post types WP core owns — skipped in the
/// unclaimed report (import handles or ignores them deliberately).
const CORE_TYPES: &[&str] = &[
    "attachment", "revision", "nav_menu_item", "customize_changeset",
    "wp_font_face", "wp_font_family", "wp_global_styles", "wp_block",
    "wp_navigation", "wp_template", "wp_template_part", "oembed_cache",
    "user_request", "wp_theme", "elementor_library",
];

/// `(dir_slug, plugin_file)` per active plugin — `active_plugins` is a
/// serialized PHP array of `dir/file.php` strings.
fn active_plugins(dump: &Path) -> Vec<(String, String)> {
    use crate::admin::bridge::playground::option_from_dump;
    let Some(raw) = option_from_dump(Some(dump), "active_plugins") else {
        eprintln!("detect: option_from_dump(active_plugins) → None");
        return Vec::new();
    };
    let items: Vec<String> = match crate::php_serialize::unserialize(raw.as_bytes()) {
        // php_serialize returns Object for assoc arrays and Array for
        // sequential ones — take values uniformly.
        Ok(serde_json::Value::Array(a)) => a.into_iter().filter_map(|i| i.as_str().map(str::to_string)).collect(),
        Ok(serde_json::Value::Object(m)) => m.into_values().filter_map(|i| i.as_str().map(str::to_string)).collect(),
        _ => {
            eprintln!("detect: active_plugins unparseable");
            Vec::new()
        }
    };
    items
        .into_iter()
        .map(|f| (f.split('/').next().unwrap_or(&f).to_string(), f))
        .collect()
}

/// Walk plugin/theme PHP files (vendor/tests skipped so fixtures can't
/// produce false positives). `hit(path, src)` sees every file under
/// `files` cap — early return `true` from `hit` stops that dir.
fn walk_php(root: &Path, files_cap: usize, mut hit: impl FnMut(&Path, &str) -> bool) {
    if !root.is_dir() {
        return;
    }
    let mut stack = vec![root.to_path_buf()];
    let mut files = 0usize;
    'dirs: while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                let n = e.file_name();
                if n != "vendor" && n != "vendor_prefixed" && n != "tests" && n != "node_modules" && n != "languages" {
                    stack.push(p);
                }
            } else if p.extension().map(|x| x == "php").unwrap_or(false) {
                files += 1;
                if files > files_cap {
                    break 'dirs;
                }
                let Ok(s) = std::fs::read_to_string(&p) else { continue };
                if hit(&p, &s) {
                    break 'dirs;
                }
            }
        }
    }
}

/// Builder markers present in a plugin dir.
fn scan_markers(dir: &Path) -> BTreeSet<&'static str> {
    let mut found = BTreeSet::new();
    walk_php(dir, 4000, |_, s| {
        for (marker, label) in BUILDER_MARKERS {
            if s.contains(marker) {
                found.insert(*label);
            }
        }
        found.len() == BUILDER_MARKERS.len()
    });
    found
}

/// post_types each plugin/theme dir registers. Two passes per file:
/// literal `register_post_type('slug'` plus dynamic forms —
/// `self::$var` / `Class::CONST` / `$var` resolved against
/// `'slug' = "..."` declarations in the SAME file (covers
/// tevily-themer's `self::$post_type` pattern). Theme dirs attribute
/// as `theme:<dir>` so theme-owned CPTs don't read as plugin work.
fn registered_types(wp_content: &Path) -> BTreeMap<String, String> {
    let re_lit = regex::Regex::new(r#"register_post_type\s*\(\s*['"]([a-z0-9_-]+)['"]"#).unwrap();
    let re_dyn = regex::Regex::new(r#"register_post_type\s*\(\s*(?:self::|static::|([A-Za-z_][A-Za-z0-9_]*)::)?\$?([A-Za-z_][A-Za-z0-9_]*)"#).unwrap();
    let mut owners: BTreeMap<String, String> = BTreeMap::new();
    /// One dir tree under `wp-content/{plugins,themes}` → owner label,
    /// literal + dynamic registrations merged.
    fn scan_dir(dir: &Path, owner: &str, re_lit: &regex::Regex, re_dyn: &regex::Regex,
                owners: &mut BTreeMap<String, String>) {
        walk_php(dir, 8000, |_, s| {
            for c in re_lit.captures_iter(s) {
                owners.entry(c[1].to_string()).or_insert_with(|| owner.to_string());
            }
            for c in re_dyn.captures_iter(s) {
                // `self::$post_type` → look for `post_type = 'slug'`
                // in the same file; `FOO::POST_TYPE`/`$var` similarly.
                // Skips function-call args (capture 2 is empty then).
                let name = c.get(2).map(|m| m.as_str()).unwrap_or("");
                if name.is_empty() {
                    continue;
                }
                let decl = regex::Regex::new(&format!(
                    r#"(?:{n}|\${n})\s*=\s*['"]([a-z0-9_-]{{2,}})['"]"#,
                    n = regex::escape(name)
                ))
                .unwrap();
                for d in decl.captures_iter(s) {
                    owners.entry(d[1].to_string()).or_insert_with(|| owner.to_string());
                }
            }
            false
        });
    }
    // read_dir order is filesystem-dependent — sort so a shared CPT
    // resolves to the same owner every run.
    let sorted = |root: std::path::PathBuf| -> Vec<std::path::PathBuf> {
        let mut v: Vec<_> = std::fs::read_dir(&root)
            .map(|rd| rd.flatten().map(|e| e.path()).collect())
            .unwrap_or_default();
        v.sort();
        v
    };
    for p in sorted(wp_content.join("plugins")) {
        if p.is_dir() {
            let owner = p.file_name().unwrap().to_string_lossy().into_owned();
            scan_dir(&p, &owner, &re_lit, &re_dyn, &mut owners);
        }
    }
    for p in sorted(wp_content.join("themes")) {
        if p.is_dir() {
            let owner = format!("theme:{}", p.file_name().unwrap().to_string_lossy());
            scan_dir(&p, &owner, &re_lit, &re_dyn, &mut owners);
        }
    }
    owners
}

/// Distinct wp post_types with row counts.
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

/// Shortcode tags used in post_content — `[tag ...]` / `[tag]`.
/// `wpcf7_contact_form` posts are skipped: their post_content is CF7
/// field notation (`[text* your-name]`), not WP shortcodes, and would
/// flood the report with form-field tags. Returns raw counts — the
/// caller filters against `registered_shortcodes`: WP itself only
/// expands registered tags, and the raw pattern also matches CSS
/// attribute selectors (`[type]`) and prose brackets that are noise
/// for the purpose of "which shortcode handlers must exist".
fn shortcodes(dump: &Path) -> BTreeMap<String, u64> {
    let prefix = crate::sqldump::detect_prefix(dump).unwrap_or_else(|_| "wp_".into());
    let table = format!("{prefix}posts");
    let re = regex::Regex::new(r#"\[\s*/?\s*([a-zA-Z_][a-zA-Z0-9_-]*)"#).unwrap();
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    let _ = crate::sqldump::rows(dump, &[table.as_str()], |_, v| {
        // column 4 = post_content, 20 = post_type.
        let pt = v.get(20).and_then(|x| x.to_string_lossy()).unwrap_or_default();
        if pt == "wpcf7_contact_form" {
            return Ok(());
        }
        if let Some(c) = v.get(4).and_then(|x| x.to_string_lossy()) {
            if c.contains('[') {
                for cap in re.captures_iter(&c) {
                    *counts.entry(cap[1].to_lowercase()).or_default() += 1;
                }
            }
        }
        Ok(())
    });
    counts
}

/// Shortcode names rpress renders natively — mirrors the match arms in
/// `render/shortcode.rs::expand_shortcodes`. Drift check: when a new arm
/// lands there, add the tag here (the names are stable strings).
const RENDERED_SHORTCODES: &[&str] = &[
    "site_reviews", "site_reviews_form", "contact-form-7",
    "gdlr_core_button", "gdlr_core_icon", "metaslider",
    "embed", "caption",
];

/// WP core shortcode tags — renderable only if rpress implements them;
/// listed separately so they don't read as plugin work.
const CORE_SHORTCODES: &[&str] = &[
    "wp_caption", "gallery", "video", "audio", "playlist", "shortcode",
];

/// Shortcode names registered via `add_shortcode('tag', ...)` in pulled
/// plugin/theme source — literal + `self::$var`/`$var` declarations in
/// the same file. Core tags live in `CORE_SHORTCODES` so they report as
/// "core", not "needs render support".
fn registered_shortcodes(wp_content: Option<&Path>) -> BTreeSet<String> {
    let mut tags: BTreeSet<String> = BTreeSet::new();
    let Some(content) = wp_content else { return tags };
    let re_lit = regex::Regex::new(r#"add_shortcode\s*\(\s*['"]([a-zA-Z0-9_-]+)['"]"#).unwrap();
    let re_dyn = regex::Regex::new(r#"add_shortcode\s*\(\s*(?:self::|static::|[A-Za-z_][A-Za-z0-9_]*::)?\$?([A-Za-z_][A-Za-z0-9_]*)"#).unwrap();
    for top in ["plugins", "themes"] {
        let Ok(rd) = std::fs::read_dir(content.join(top)) else { continue };
        for e in rd.flatten() {
            let dir = e.path();
            if !dir.is_dir() {
                continue;
            }
            walk_php(&dir, 8000, |_, s| {
                for c in re_lit.captures_iter(s) {
                    tags.insert(c[1].to_lowercase());
                }
                for c in re_dyn.captures_iter(s) {
                    let name = c.get(1).map(|m| m.as_str()).unwrap_or("");
                    if name.is_empty() {
                        continue;
                    }
                    let decl = regex::Regex::new(&format!(
                        r#"(?:{n}|\${n})\s*=\s*['"]([a-zA-Z0-9_-]{{2,}})['"]"#,
                        n = regex::escape(name)
                    ))
                    .unwrap();
                    for d in decl.captures_iter(s) {
                        tags.insert(d[1].to_lowercase());
                    }
                }
                false
            });
        }
    }
    tags
}

/// wp_options name → count, grouped by a 2-segment prefix
/// (`tourmaster_general` → `tourmaster`, `trustindex_tripadvisor_lang` →
/// `trustindex_tripadvisor`). WP core keys and transients fold into
/// `wp-core/misc` so the report shows signal, not 200 one-off rows.
fn option_prefix_counts(dump: &Path) -> BTreeMap<String, u64> {
    let prefix = crate::sqldump::detect_prefix(dump).unwrap_or_else(|_| "wp_".into());
    let table = format!("{prefix}options");
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    let core = |k: &str| -> bool {
        let k = k.trim_start_matches('_');
        const CORE: &[&str] = &[
            "transient", "site_transient", "cron", "widget_", "theme_mods_",
            "rewrite_rules", "recently_activated", "uninstall_plugins",
            "recovery_", "db_upgraded", "https_", "auto_", "siteurl", "home",
            "blog", "admin_", "users_", "default_", "permalink", "gmt_offset",
            "upload_", "posts_", "links_", "comment", "thumbnail_", "medium",
            "large_", "image_", "page_", "show_", "thread_", "time",
            "use_", "require_", "moderation", "disallowed", "blacklist",
            "close_", "active_", "stylesheet", "template", "date_",
            "start_of_week", "WPLANG", "mailserver", "ping_", "rss_",
            "avatar", "emoji", "html_type", "sticky_posts", "category_",
            "tag_", "sidebars_widgets", "user_count", "new_admin_email",
            "wp_user_roles", "wp_deferred", "wp_attachment", "wp_force",
            "wp_page", "wp_notes", "wp_calendar", "wp_batch", "wp_email",
            "wp_installation", "wp_sitemap", "wp_https", "wp_user", "wp_",
        ];
        CORE.iter().any(|p| k.starts_with(p))
    };
    let _ = crate::sqldump::rows(dump, &[table.as_str()], |_, v| {
        let Some(k) = v.get(1).and_then(|x| x.to_string_lossy()) else { return Ok(()) };
        let pref = if core(&k) {
            "wp-core/misc".to_string()
        } else {
            // First underscore segment is the plugin namespace —
            // `elementor_*`, `tribe_*`, `babe_*` each collapse to one
            // row; two-segment names would shatter them into ~40 rows.
            k.trim_start_matches('_').split(|c| c == '_' || c == '-').next().unwrap_or("").to_string()
        };
        *counts.entry(pref).or_default() += 1;
        Ok(())
    });
    counts
}

/// Option prefixes covered by adapter `option_prefixes` (the same
/// source import uses to whitelist `settings` rows). `k` is the
/// first-segment group (`babe`, `tribe`) — compare with and without the
/// trailing underscore since adapter prefixes carry both styles.
fn covered_prefix(k: &str) -> Option<&'static str> {
    if k == "wp-core/misc" {
        return Some("core");
    }
    let with_us = format!("{k}_");
    for a in crate::plugins::adapters() {
        if let Some(h) = a.import_hooks() {
            if h.option_prefixes.iter().any(|p| k.starts_with(p) || with_us.starts_with(p)) {
                return Some(a.slug());
            }
        }
    }
    None
}

/// The wp post_types already claimed: adapter claim_type is a fn, so
/// probe every observed type rather than inverting the fns.
fn claimed<'a>(types: impl Iterator<Item = &'a String>) -> BTreeSet<String> {
    let mut out: BTreeSet<String> = CORE_TYPES.iter().map(|s| s.to_string()).collect();
    for t in ["post", "page", "site-review", "custom_css", "wpcf7_contact_form"] {
        out.insert(t.to_string());
    }
    for t in types {
        for a in crate::plugins::adapters() {
            if a.import_hooks().and_then(|h| (h.claim_type)(t)).is_some() {
                out.insert(t.clone());
            }
        }
    }
    out
}

/// Directories present under wp-content/{plugins,themes} (inactive
/// plugins/themes still ship code + CPT registrations the report uses).
fn dir_names(dir: &Path) -> BTreeSet<String> {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}


pub fn run(dump: &Path, www: Option<&Path>) -> Result<()> {
    if !dump.is_file() {
        anyhow::bail!("dump not found: {}", dump.display());
    }
    use crate::admin::bridge::playground::option_from_dump;
    let content = www.map(|w| w.join("wp-content"));
    let plugins_dir = content.as_ref().map(|c| c.join("plugins"));
    let themes_dir = content.as_ref().map(|c| c.join("themes"));

    println!("== theme ==");
    match option_from_dump(Some(dump), "stylesheet") {
        Some(ss) => {
            let parent = option_from_dump(Some(dump), "template").unwrap_or_default();
            match crate::plugins::theme_for_stylesheet(&ss)
                .or_else(|| crate::plugins::themes::theme_for_stylesheet(&ss))
            {
                Some(_) => println!("  {ss} (parent: {parent}) → ThemeDef matched — chrome/import covered"),
                None => println!("  {ss} (parent: {parent}) → NO ThemeDef — menus/chrome fall back to DEFAULT_THEME; consider a theme spec"),
            }
        }
        None => println!("  no stylesheet option in dump"),
    }
    let themes_on_disk = themes_dir.as_deref().map(dir_names).unwrap_or_default();
    if !themes_on_disk.is_empty() {
        println!("  themes on disk: {}", themes_on_disk.iter().cloned().collect::<Vec<_>>().join(", "));
    }

    let active = active_plugins(dump);
    let active_dirs: BTreeSet<String> = active.iter().map(|(d, _)| d.clone()).collect();
    let on_disk = plugins_dir.as_deref().map(dir_names).unwrap_or_default();
    let owners = content.as_deref().map(registered_types).unwrap_or_default();

    println!("\n== active plugins ==");
    let mut php_candidates: Vec<String> = Vec::new();
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
        let cpts: Vec<&String> = owners
            .iter()
            .filter(|(_, o)| *o == dir_slug)
            .map(|(t, _)| t)
            .collect();
        let cpt_note = if cpts.is_empty() {
            String::new()
        } else {
            format!(" · registers {}", cpts.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(","))
        };
        match known {
            Some(a) => {
                let backend = format!("{:?}", a.backend());
                println!("  {file} → adapter '{}' ({backend}){cpt_note}", a.slug());
                if backend == "PhpBridge" {
                    php_candidates.push(a.slug().to_string());
                }
            }
            None if !markers.is_empty() => {
                println!("  {file} → NO adapter — markers: {}{cpt_note}", markers.iter().cloned().collect::<Vec<_>>().join(", "));
                println!("      tier-3 candidate: needs a PhpBridge adapter (query-forward + plugin-activation filter + write_specs)");
                php_candidates.push(format!("<{dir_slug}: adapter needed>"));
            }
            None => {
                println!("  {file} → NO adapter — data-only{cpt_note}");
            }
        }
    }

    let inactive: Vec<&String> = on_disk.iter().filter(|d| !active_dirs.contains(*d)).collect();
    if !inactive.is_empty() {
        println!("\n== on disk but inactive ==");
        for d in inactive {
            let cpts: Vec<&String> = owners.iter().filter(|(_, o)| *o == d).map(|(t, _)| t).collect();
            println!("  {d}{}", if cpts.is_empty() { String::new() } else { format!(" · registers {}", cpts.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(",")) });
        }
    }

    let types = post_types(dump);
    let claimed_set = claimed(types.keys());
    println!("\n== post types ==");
    let mut unclaimed: Vec<(String, u64, Option<String>)> = Vec::new();
    for (t, n) in &types {
        if CORE_TYPES.contains(&t.as_str()) {
            continue;
        }
        let owner = owners.get(t).cloned();
        let owner_note = owner.as_ref().map(|o| format!(" (registered by {o})")).unwrap_or_default();
        if claimed_set.contains(t) {
            println!("  {t} ({n}) → claimed{owner_note}");
        } else {
            println!("  {t} ({n}) → UNCLAIMED{owner_note} — falls to generic post/page or is skipped");
            unclaimed.push((t.clone(), *n, owner));
        }
    }

    // WP only expands registered shortcodes — raw `[...]` matches also
    // catch CSS attribute selectors and prose brackets, so unregistered
    // names collapse into one line instead of flooding the report.
    let sc = shortcodes(dump);
    if !sc.is_empty() {
        let known = registered_shortcodes(content.as_deref());
        let mut other_n = 0u64;
        let mut other_k = 0usize;
        println!("\n== shortcodes in post_content ==");
        for (tag, n) in sc.iter() {
            if RENDERED_SHORTCODES.contains(&tag.as_str()) {
                println!("  [{tag}] ×{n} → rpress renders natively");
            } else if CORE_SHORTCODES.contains(&tag.as_str()) {
                println!("  [{tag}] ×{n} → wp core tag — unexpanded unless rpress handles it");
            } else if known.contains(tag.as_str()) {
                println!("  [{tag}] ×{n} → registered in plugin/theme source — needs render support");
            } else {
                other_k += 1;
                other_n += n;
            }
        }
        if other_k > 0 {
            println!("  … {other_k} unregistered bracket patterns ({other_n} hits) — CSS/prose noise or a shortcode whose source isn't pulled");
        }
    }

    let opts = option_prefix_counts(dump);
    if !opts.is_empty() {
        println!("\n== wp_options coverage (by prefix) ==");
        for (pref, n) in &opts {
            match covered_prefix(pref) {
                Some(slug) => println!("  {pref}_* ({n}) → imported under '{slug}' whitelist"),
                None => println!("  {pref}_* ({n}) → not whitelisted"),
            }
        }
    }

    println!("\n== suggested next steps ==");
    if !php_candidates.is_empty() {
        println!("  PLUGINS_PHP={}", php_candidates.join(","));
    }
    for (t, n, owner) in &unclaimed {
        let owner = owner.as_deref().unwrap_or("<plugin-slug>");
        println!("  unclaimed type {t} ({n} rows) → plugins/specs/{owner}.toml: claim_types += [\"{t}\"]");
    }
    println!("\nNOTE: attribution comes from register_post_type/option prefixes/shortcode tags in the dump+pulled source — structural facts. `Backend` choice (Rust port vs Data vs PhpBridge) still needs a human decision.");
    Ok(())
}
