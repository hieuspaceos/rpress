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
/// produce false positives). `hit(path, src)` sees every PHP file in the
/// tree — early return `true` from `hit` stops that dir.
fn walk_php(root: &Path, mut hit: impl FnMut(&Path, &str) -> bool) {
    if !root.is_dir() {
        return;
    }
    let mut stack = vec![root.to_path_buf()];
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
                let Ok(s) = std::fs::read_to_string(&p) else { continue };
                if hit(&p, &s) {
                    break 'dirs;
                }
            }
        }
    }
}

/// Builder markers present in a plugin dir, with file counts and
/// whether any hit sits in a top-level plugin file. A single `Widget_Base`
/// string in one bundled shim is NOT a builder — aucoeur showed
/// code-snippets/ez-toc/presto/simply-static false-positiving that way;
/// a real builder hits its marker in ≥2 files or in `<dir>/<file>.php`
/// at the plugin root.
fn scan_markers(dir: &Path) -> BTreeMap<&'static str, (usize, bool)> {
    let mut found: BTreeMap<&'static str, (usize, bool)> = BTreeMap::new();
    walk_php(dir, |p, s| {
        let top_level = p
            .strip_prefix(dir)
            .map(|r| r.components().count() == 1)
            .unwrap_or(false);
        for (marker, label) in BUILDER_MARKERS {
            if s.contains(marker) {
                let e = found.entry(*label).or_default();
                e.0 += 1;
                e.1 |= top_level;
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
        walk_php(dir, |_, s| {
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

/// WP core shortcode tags rpress's own `expand_shortcodes` handles —
/// adapter tags are declared on each adapter's `rendered_shortcodes`.
const CORE_HANDLED: &[&str] = &["embed", "caption"];

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
            walk_php(&dir, |_, s| {
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

/// The wp post_types already claimed: `import::kind_for_post_type` is
/// the single authority — adapter claims, core post/page, and
/// hard-coded storage kinds (ml-*) all live there. Core internals
/// (attachment, revision, wpcf7_contact_form, custom_css…) are
/// claimed separately in `CORE_TYPES`/`IMPORTED_BUILTINS` since
/// `kind_for_post_type` returns `None` for them by design.
const IMPORTED_BUILTINS: &[&str] = &[
    "custom_css", "wpcf7_contact_form",
];
fn claimed<'a>(types: impl Iterator<Item = &'a String>) -> BTreeSet<String> {
    let mut out: BTreeSet<String> = CORE_TYPES
        .iter()
        .chain(IMPORTED_BUILTINS)
        .map(|s| s.to_string())
        .collect();
    for t in types {
        if crate::import::kind_for_post_type(t).is_some() {
            out.insert(t.clone());
        }
    }
    out
}

/// wp `post_type`s adapters deliberately skip (`ignore_types`) — a
/// documented skip, so they leave the unclaimed suggestion list too.
fn ignored<'a>(types: impl Iterator<Item = &'a String>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for t in types {
        for a in crate::plugins::adapters() {
            if a.ignore_types().iter().any(|i| *i == t.as_str()) {
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

    let mut theme_missing: Option<String> = None;
    println!("== theme ==");
    match option_from_dump(Some(dump), "stylesheet") {
        Some(ss) => {
            let parent = option_from_dump(Some(dump), "template").unwrap_or_default();
            match crate::plugins::theme_for_stylesheet(&ss)
                .or_else(|| crate::plugins::themes::theme_for_stylesheet(&ss))
            {
                Some(_) => println!("  {ss} (parent: {parent}) → ThemeDef matched — chrome/import covered"),
                None => {
                    println!("  {ss} (parent: {parent}) → NO ThemeDef — menus/chrome fall back to DEFAULT_THEME; consider a theme spec");
                    theme_missing = Some(ss);
                }
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
        // tier-3 evidence = a marker hit in a top-level plugin file.
        // Bundled integration shims (inc/Integrations/Elementor/…) are
        // the classic false positive — a real builder's marker reaches
        // its main file.
        // strong = marker in ≥3 files (real framework usage —
        // elementor hits 68), or in the main <slug>.php, or combined
        // with the plugin registering CPTs (runtime behavior like
        // tevily-themer's elementor/el-widgets/ base).
        let strong: Vec<&'static str> = markers
            .iter()
            .filter(|(_, (n, top))| *n >= 3 || *top)
            .map(|(l, _)| *l)
            .collect();
        let marker_note = if markers.is_empty() {
            String::new()
        } else {
            format!(" — markers: {}", markers.keys().cloned().collect::<Vec<_>>().join(", "))
        };
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
            None if !strong.is_empty() || (!markers.is_empty() && !cpts.is_empty()) => {
                println!("  {file} → NO adapter{marker_note}{cpt_note}");
                println!("      tier-3 candidate: needs a PhpBridge adapter (query-forward + plugin-activation filter + write_specs)");
                php_candidates.push(format!("<{dir_slug}: adapter needed>"));
            }
            None => {
                println!("  {file} → NO adapter — data-only{marker_note}{cpt_note}");
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
    if !plugins_dir.as_deref().is_some_and(|d| d.is_dir()) {
        println!("\n  note: wp-content/plugins not pulled — CPT-registration and builder-marker attribution unavailable; pass --www with a pulled wp-content to enrich");
    }
    if content.is_some() && !themes_dir.as_deref().is_some_and(|d| d.is_dir()) {
        println!("  note: wp-content/themes not pulled — theme-registered CPTs unattributed");
    }

    let types = post_types(dump);
    let claimed_set = claimed(types.keys());
    let ignored_set = ignored(types.keys());
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
        } else if ignored_set.contains(t) {
            println!("  {t} ({n}) → ignored by spec{owner_note} — storage CPT, skipped at import");
        } else {
            println!("  {t} ({n}) → UNCLAIMED{owner_note} — falls to generic post/page or is skipped");
            unclaimed.push((t.clone(), *n, owner));
        }
    }

    // WP only expands registered shortcodes — raw `[...]` matches also
    // catch CSS attribute selectors and prose brackets, so unregistered
    // names collapse into one line instead of flooding the report.
    let sc = shortcodes(dump);
    let known = if sc.is_empty() { BTreeSet::new() } else { registered_shortcodes(content.as_deref()) };
    if !sc.is_empty() {
        let mut other_n = 0u64;
        let mut other_k = 0usize;
        println!("\n== shortcodes in post_content ==");
        let rendered: BTreeSet<&'static str> = crate::plugins::adapters()
            .iter()
            .flat_map(|a| a.rendered_shortcodes().iter().copied())
            .collect();
        for (tag, n) in sc.iter() {
            if rendered.contains(tag.as_str()) || CORE_HANDLED.contains(&tag.as_str()) {
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

    // Routing board: each finding lands on the exact rpress artifact
    // that owns the fix. detect's job ends at naming the file — the
    // Backend choice stays a human decision.
    println!("\n== routing — where each finding lands ==");
    let mut routed = false;
    if let Some(ss) = &theme_missing {
        println!("  THEME  {ss} → themes/specs/{ss}.toml (+ templates/theme/{ss}/base_<lang>.html) — else DEFAULT_THEME chrome");
        routed = true;
    }
    if !php_candidates.is_empty() {
        println!("  BRIDGE sites/<site>.env: PLUGINS_PHP={}", php_candidates.join(","));
        routed = true;
    }
    let rendered2: BTreeSet<&'static str> = crate::plugins::adapters()
        .iter()
        .flat_map(|a| a.rendered_shortcodes().iter().copied())
        .collect();
    if !sc.is_empty() {
        let need_render: Vec<&String> = sc
            .keys()
            .filter(|t| !rendered2.contains(t.as_str()) && !CORE_HANDLED.contains(&t.as_str())
                && !CORE_SHORTCODES.contains(&t.as_str()) && known.contains(t.as_str()))
            .collect();
        if !need_render.is_empty() {
            println!("  RENDER {} → render/shortcode.rs arm (+ owning adapter's rendered_shortcodes)", need_render.iter().map(|t| format!("[{t}]")).collect::<Vec<_>>().join(" "));
            routed = true;
        }
    }
    let unattributed = unclaimed.iter().filter(|(_, _, o)| o.is_none()).count();
    if unattributed > 0 && !plugins_dir.as_deref().is_some_and(|d| d.is_dir()) {
        println!("  PULL   {unattributed} unclaimed types show <plugin-slug> — pull wp-content/plugins to name the spec file");
    }
    for (t, n, owner) in &unclaimed {
        let owner = owner.as_deref().unwrap_or("<plugin-slug>");
        println!("  CLAIM  {t} ({n} rows) → plugins/specs/{owner}.toml claim_types, or wp-internals.toml ignore_types if it's storage");
        routed = true;
    }
    let unwhitelisted: Vec<&String> = opts
        .keys()
        .filter(|p| covered_prefix(p).is_none() && p.as_str() != "wp-core/misc")
        .collect();
    if !unwhitelisted.is_empty() {
        let n: u64 = unwhitelisted.iter().map(|p| opts[*p]).sum();
        println!("  IMPORT {} option prefixes ({n} rows) unwhitelisted → owning adapter's option_prefixes (plugins/specs/*.toml) or accept as loss", unwhitelisted.len());
        routed = true;
    }
    if !routed {
        println!("  nothing pending — this site's surface is fully covered");
    }
    println!("\nNOTE: attribution comes from register_post_type/option prefixes/shortcode tags in the dump+pulled source — structural facts. `Backend` choice (Rust port vs Data vs PhpBridge) still needs a human decision.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Theme-dir CPT attribution: a `register_post_type` in the theme's
    /// own PHP resolves to `theme:<dir>`, not a plugin slug — the sweep
    /// had no theme-CPT site so this is the only coverage of that path.
    #[test]
    fn theme_registers_attribute_as_theme_dir() {
        let root = std::env::temp_dir().join(format!("rpress-detect-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let theme = root.join("themes/fake-theme");
        std::fs::create_dir_all(&theme).unwrap();
        std::fs::write(
            theme.join("functions.php"),
            "<?php register_post_type('theme_faq', ['label'=>'FAQ']);",
        )
        .unwrap();
        let owners = registered_types(&root);
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(owners.get("theme_faq").map(|s| s.as_str()), Some("theme:fake-theme"));
    }

    /// Dynamic `self::$var` registrations resolve against same-file
    /// declarations — the ba-book-everything pattern the audit caught.
    #[test]
    fn dynamic_self_var_registration_resolves() {
        let root = std::env::temp_dir().join(format!("rpress-detect-dyn-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let plug = root.join("plugins/fake-plug");
        std::fs::create_dir_all(&plug).unwrap();
        std::fs::write(
            plug.join("types.php"),
            "<?php class T { static $faq_post_type = 'faq'; function r() { register_post_type(self::$faq_post_type, []); } }",
        )
        .unwrap();
        let owners = registered_types(&root);
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(owners.get("faq").map(|s| s.as_str()), Some("fake-plug"));
    }

    /// claim wins over ignore: core-claimed storage kinds can't sit in
    /// `ignore_types` without the report lying.
    #[test]
    fn ml_types_claim_not_ignore() {
        assert_eq!(crate::import::kind_for_post_type("ml-slider"), Some("ml-slider"));
        for a in crate::plugins::adapters() {
            assert!(!a.ignore_types().contains(&"ml-slider"), "{} ignores a core-claimed type", a.slug());
        }
    }
}
