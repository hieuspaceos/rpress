use rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct LangConfig {
    pub primary: String,
    pub langs: Vec<String>,
}

pub fn config(conn: &Connection) -> LangConfig {
    let mut langs: Vec<String> = setting(conn, "languages")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    if langs.is_empty() {
        if let Ok(mut stmt) = conn.prepare("SELECT DISTINCT lang FROM posts WHERE lang<>'' ORDER BY CASE WHEN lang='fr' THEN 0 ELSE 1 END, lang") {
            if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                langs = rows.filter_map(Result::ok).collect();
            }
        }
    }
    if langs.is_empty() {
        langs.push("fr".to_string());
    }
    let primary = setting(conn, "primary_lang");
    let primary = if primary.is_empty() { langs[0].clone() } else { primary };
    LangConfig { primary, langs }
}

pub fn setting(conn: &Connection, key: &str) -> String {
    conn.query_row("SELECT value FROM settings WHERE key=?1", [key], |r| r.get::<_, String>(0))
        .unwrap_or_default()
}

pub fn prefix_for(cfg: &LangConfig, lang: &str) -> String {
    if lang.is_empty() || lang == cfg.primary { String::new() } else { lang.to_string() }
}

pub fn url_prefix(cfg: &LangConfig, lang: &str) -> String {
    let p = prefix_for(cfg, lang);
    if p.is_empty() { String::new() } else { format!("/{p}") }
}

pub fn rel_prefix(cfg: &LangConfig, lang: &str) -> String {
    prefix_for(cfg, lang)
}

pub fn split_path<'a>(cfg: &LangConfig, clean: &'a str) -> (String, &'a str) {
    for lang in cfg.langs.iter().filter(|l| *l != &cfg.primary) {
        let prefix = format!("{lang}/");
        if let Some(rest) = clean.strip_prefix(&prefix) {
            return (lang.clone(), rest);
        }
        if clean == lang {
            return (lang.clone(), "");
        }
    }
    (cfg.primary.clone(), clean)
}

/// Canonicalize signal for the primary-language prefix: `/fr/<rest>` on an
/// fr-primary site is a duplicate of `/<rest>` — return `rest` so the
/// handler can 301 to the canonical URL. `None` for primary-less and
/// non-primary-prefixed paths (split_path handles those).
pub fn primary_alias<'a>(cfg: &LangConfig, clean: &'a str) -> Option<&'a str> {
    if cfg.primary.is_empty() {
        return None;
    }
    let prefix = format!("{}/", cfg.primary);
    if let Some(rest) = clean.strip_prefix(&prefix) {
        return Some(rest);
    }
    if clean == cfg.primary {
        return Some("");
    }
    None
}

pub fn page_path(cfg: &LangConfig, lang: &str, _kind: &str, slug: &str, kind_base: Option<&str>) -> String {
    let mut parts: Vec<String> = Vec::new();
    let lp = rel_prefix(cfg, lang);
    if !lp.is_empty() { parts.push(lp); }
    // Plugin kinds serve under a URL base segment resolved via KindBases
    // (today only the tour adapter; root kinds pass None).
    if let Some(b) = kind_base {
        if !b.is_empty() { parts.push(b.trim_matches('/').to_string()); }
    }
    if !slug.is_empty() { parts.push(slug.trim_matches('/').to_string()); }
    if parts.is_empty() { "/".to_string() } else { format!("/{}/", parts.join("/")) }
}

pub fn template_for(lang: &str) -> String {
    format!("base_{lang}.html")
}

/// Resolve the base template filename for `lang`: exact `base_{lang}`
/// → primary-lang base → any base_* in the probe dirs. `theme_dir`
/// (adapter `template_dir`, may be `None`) is probed before the shared
/// root — the returned name is dir-relative so `{% extends base %}` and
/// the loader resolve it the same way.
pub fn template_name(
    templates: &std::path::Path,
    theme_dir: Option<&std::path::Path>,
    cfg: &LangConfig,
    lang: &str,
) -> String {
    // The matched theme's dir is authoritative: try its bases first, and
    // never fall ACROSS to a different theme's dir (a missing
    // base_{lang} inside one theme dir means that theme wants the
    // primary-lang fallback, not another theme's chrome). The shared
    // root participates only when the theme dir itself has no bases
    // (theme_template_dir already resolves that case to None).
    let theme_has_bases = theme_dir
        .map(|d| {
            std::fs::read_dir(d)
                .map(|rd| {
                    rd.flatten().any(|e| {
                        e.file_name().to_string_lossy().starts_with("base_")
                            && e.path().is_file()
                    })
                })
                .unwrap_or(false)
        })
        .unwrap_or(false);
    let dirs: Vec<&std::path::Path> = if theme_has_bases {
        theme_dir.into_iter().collect()
    } else {
        [theme_dir, Some(templates)].into_iter().flatten().collect()
    };
    for name in [template_for(lang), template_for(&cfg.primary), "base_fr.html".into(), "base_es.html".into(), "base_en.html".into()] {
        if dirs.iter().any(|d| d.join(&name).exists()) {
            return name;
        }
    }
    template_for(lang)
}

pub fn page_url(cfg: &LangConfig, site: &str, lang: &str, kind: &str, slug: &str, kind_base: Option<&str>) -> String {
    format!("{}{}", site.trim_end_matches('/'), page_path(cfg, lang, kind, slug, kind_base))
}

/// Flag asset filename for a language code. The vendored Polylang flag set
/// is country-coded (ISO-3166), not language-coded (ISO-639) — `en` renders
/// `us.svg`, `vi` renders `vn.svg`, and `en.svg`/`vi.svg` don't exist.
/// Fallback is the lang code itself (works for fr/es/de/nl/pt/th/… whose
/// codes coincide).
pub fn flag_file(lang: &str) -> String {
    let code = match lang {
        "en" => "us",
        "vi" => "vn",
        "ja" => "jp",
        "zh" | "zh-cn" | "zh-hans" => "cn",
        "zh-tw" | "zh-hant" => "tw",
        "ko" => "kr",
        "el" => "gr",
        "he" => "il",
        "uk" => "ua",
        "da" => "dk",
        "cs" => "cz",
        "et" => "ee",
        "fa" => "ir",
        "hi" => "in",
        "hy" => "am",
        "ka" => "ge",
        "ms" => "my",
        "nb" | "nn" => "no",
        "sl" => "si",
        "sq" => "al",
        "sv" => "se",
        "ur" => "pk",
        "ar" => "ar.lang", // Polylang ships ar.lang.svg for Arabic
        l => l,
    };
    format!("{code}.svg")
}

/// BCP-47-ish locale for `lang` (`fr`→`fr-FR`, `es`→`es-ES`, `en`→`en-US`).
/// Unknown codes map to `{lang}-{lang upper}` which is the WP locale shape
/// for most languages (de-DE, it-IT, pt-PT).
pub fn html_lang(lang: &str) -> String {
    match lang {
        "en" => "en-US".into(),
        "es" => "es-ES".into(),
        "fr" => "fr-FR".into(),
        other => format!("{other}-{}", other.to_uppercase()),
    }
}

/// Capitalized month names for `date_i18n('F Y')`-style output (tour_search
/// 'month' combobox). French is the fallback — the reference site emits it
/// for the primary language and the combobox is a UI nicety, not a locale
/// guarantee.
pub fn month_names(lang: &str) -> &'static [&'static str; 12] {
    const FR: [&str; 12] = [
        "Janvier", "Février", "Mars", "Avril", "Mai", "Juin",
        "Juillet", "Août", "Septembre", "Octobre", "Novembre", "Décembre",
    ];
    const ES: [&str; 12] = [
        "Enero", "Febrero", "Marzo", "Abril", "Mayo", "Junio",
        "Julio", "Agosto", "Septiembre", "Octubre", "Noviembre", "Diciembre",
    ];
    const EN: [&str; 12] = [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December",
    ];
    const DE: [&str; 12] = [
        "Januar", "Februar", "März", "April", "Mai", "Juni",
        "Juli", "August", "September", "Oktober", "November", "Dezember",
    ];
    match lang {
        "es" => &ES,
        "en" => &EN,
        "de" => &DE,
        _ => &FR,
    }
}

/// Theme UI strings keyed by name — the N-lang replacement for the old
/// `if es { .. } else { .. }` literals in chrome_vars/reading_time.
/// Fallback is French (the reference primary).
pub fn t(lang: &str, key: &str) -> &'static str {
    let table: &[(&str, &str)] = match lang {
        "es" => &[
            ("written_by", "Escrito por"),
            ("reading_time", "Tiempo de lectura"),
            ("under_minute", "Menos de un minuto"),
            ("minutes", "minutos"),
            ("from", "Desde"),
        ],
        "en" => &[
            ("written_by", "Written by"),
            ("reading_time", "Reading time"),
            ("under_minute", "Under a minute"),
            ("minutes", "minutes"),
            ("from", "From"),
        ],
        "de" => &[
            ("written_by", "Geschrieben von"),
            ("reading_time", "Lesezeit"),
            ("under_minute", "Weniger als eine Minute"),
            ("minutes", "Minuten"),
            ("from", "Ab"),
        ],
        _ => &[
            ("written_by", "Écrit par"),
            ("reading_time", "Temps de lecture"),
            ("under_minute", "Moins d’une minute"),
            ("minutes", "minutes"),
            ("from", "De"),
        ],
    };
    table
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
        .unwrap_or("")
}
