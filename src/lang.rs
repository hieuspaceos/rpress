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

pub fn page_path(cfg: &LangConfig, lang: &str, kind: &str, slug: &str, tour_base: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    let lp = rel_prefix(cfg, lang);
    if !lp.is_empty() { parts.push(lp); }
    if kind == "tour" { parts.push(tour_base.trim_matches('/').to_string()); }
    if !slug.is_empty() { parts.push(slug.trim_matches('/').to_string()); }
    if parts.is_empty() { "/".to_string() } else { format!("/{}/", parts.join("/")) }
}

pub fn template_for(lang: &str) -> String {
    format!("base_{lang}.html")
}

pub fn template_name(templates: &std::path::Path, cfg: &LangConfig, lang: &str) -> String {
    let exact = template_for(lang);
    if templates.join(&exact).exists() {
        return exact;
    }
    let primary = template_for(&cfg.primary);
    if templates.join(&primary).exists() {
        return primary;
    }
    if templates.join("base_fr.html").exists() {
        return "base_fr.html".to_string();
    }
    "base_es.html".to_string()
}

pub fn page_url(cfg: &LangConfig, site: &str, lang: &str, kind: &str, slug: &str, tour_base: &str) -> String {
    format!("{}{}", site.trim_end_matches('/'), page_path(cfg, lang, kind, slug, tour_base))
}
