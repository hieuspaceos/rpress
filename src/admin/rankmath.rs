//! Rank Math-style SEO metabox — 4 tabs (General / Advanced / Schema /
//! Social) rendered inside the post editor. Fields are gathered into a
//! hidden `rank_math` JSON input at submit; `crud::save_rankmath` persists
//! them to `post_meta` in WP's own shapes (PHP-serialized robots array,
//! `rank_math_primary_{TaxName}` keys, `rank_math_schema_{Type}`).

use super::{esc, need_auth, site_url, tax_label, App, SiteCtx, Sess};
use crate::db as crud;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use rusqlite::Connection;
use std::collections::{BTreeMap, HashMap};

/// Canonical `rank_math_primary_{X}` suffix per taxonomy slug — from the
/// live-data audit: WP registered the tour taxonomies capitalized/accented
/// (`Durée`, `Région`, `Activites`, `Pays`, `Profil`, `Style`); builtin
/// taxonomies stay lowercase (`category`, `tour_category`).
fn primary_key_default(tax: &str) -> String {
    match tax {
        "duree" => "Durée".into(),
        "profil" => "Profil".into(),
        "region" => "Région".into(),
        "style" => "Style".into(),
        other => other.into(),
    }
}

/// Called from `crud::save_post` inside the post-save transaction: rebuilds
/// the taxonomy→meta-key map for this post and delegates to
/// `crud::save_rankmath`, so SEO metas commit or roll back with the post.
pub(crate) fn save_conn(conn: &Connection, id: i64, f: &crud::PostForm) -> Result<(), String> {
    crud::save_rankmath(conn, id, f, &tax_map(conn, id)).map_err(|e| e.to_string())
}

/// Inner markup of the "SEO Details" cell — score pill, focus keyword,
/// link counts (no `<td>` wrapper). Rendered by the lazy
/// `/admin/seo/cell/{id}` fragment endpoint below.
fn seo_inner(a: &crud::SeoAnalysis) -> String {
    let (cls, txt) = if a.keyword.is_empty() {
        ("rm-seo-na", "N/A".to_string())
    } else {
        let cls = if a.score >= 80 {
            "rm-seo-good"
        } else if a.score >= 50 {
            "rm-seo-ok"
        } else {
            "rm-seo-bad"
        };
        (cls, format!("{} / 100", a.score))
    };
    let kw = if a.keyword.is_empty() {
        "Not Set".to_string()
    } else {
        a.keyword.clone()
    };
    format!(
        r#"<div class="rm-seo-pill {cls}">{txt}</div><div class="rm-seo-kw">Keyword: {kw}</div><div class="rm-seo-links">Links: 🔗 {i} | ↗ {e} | ↩ {b}</div>"#,
        i = a.links_internal,
        e = a.links_external,
        b = a.links_inbound,
    )
}

/// `GET /admin/seo/cell/{id}` — lazy fill for the list-table SEO column.
/// The list page renders a `<td data-seo-id>` placeholder and a script
/// swaps in this fragment, keeping `seo_analysis` (≈7 queries/row) off
/// the list render path. Returns the cell's *inner* HTML only.
pub(crate) async fn seo_cell_frag(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    let Ok((_s, sc)) = need_auth(&app, &headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let body = match crate::db::conn(&sc.db) {
        Ok(c) => seo_inner(&crud::seo_analysis(&c, id, &site_url(sc), None, None, None, None)),
        Err(_) => String::new(),
    };
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], body).into_response()
}

/// Rank Math analysis accordion inner HTML (Basic/Additional/Title/Content
/// groups + link summary) — built by `metabox_html` lazily via the frag
/// endpoint so `seo_analysis`'s inbound-links scan stays off edit-page render.
fn analysis_inner(a: &crud::SeoAnalysis) -> String {
    let checks_html = |checks: &[crud::SeoCheck]| -> String {
        checks
            .iter()
            .map(|c| {
                let (icon, cls) = match c.pass {
                    Some(true) => ("✓", "rm-pass"),
                    Some(false) => ("✗", "rm-fail"),
                    None => ("•", ""),
                };
                format!("<li class=\"{cls}\"><span class=\"rm-ic\">{icon}</span> {}</li>", esc(&c.label))
            })
            .collect()
    };
    let group = |title: &str, checks: &[crud::SeoCheck], open: bool| -> String {
        let errors = checks.iter().filter(|c| c.pass == Some(false)).count();
        let badge = if errors == 0 && checks.iter().all(|c| c.pass.is_some()) {
            "<span class=\"rm-gpill good\">✓ All Good</span>".to_string()
        } else if errors > 0 {
            format!("<span class=\"rm-gpill bad\">✗ {errors} Error{}</span>", if errors > 1 { "s" } else { "" })
        } else {
            "<span class=\"rm-gpill\">—</span>".to_string()
        };
        format!(
            "<div class=\"rm-group{}\"><div class=\"rm-ghead\">{title} {badge}<span class=\"rm-arrow\">{}</span></div><ul class=\"rm-glist\">{}</ul></div>",
            if open { " on" } else { "" },
            if open { "⌃" } else { "⌄" },
            checks_html(checks)
        )
    };
    format!(
        "{}<div class=\"rm-summary\">{} internal link(s) · {} external · {} inbound — {} words</div>",
        [
            group("Basic SEO", &a.basic, true),
            group("Additional", &a.additional, false),
            group("Title Readability", &a.title_readability, false),
            group("Content Readability", &a.content_readability, false),
        ]
        .join(""),
        a.links_internal, a.links_external, a.links_inbound, a.words
    )
}

/// `GET /admin/seo/analysis/{id}` — lazy fill for the metabox accordion.
pub(crate) async fn seo_analysis_frag(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    let Ok((_s, sc)) = need_auth(&app, &headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let body = match crate::db::conn(&sc.db) {
        Ok(c) => analysis_inner(&crud::seo_analysis(&c, id, &site_url(sc), None, None, None, None)),
        Err(_) => String::new(),
    };
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], body).into_response()
}

/// (taxonomy_slug, meta_key_suffix) for each taxonomy the post uses.
/// If a `rank_math_primary_*` row already exists for the post under a
/// candidate key, that key wins so reads/writes stay consistent.
fn tax_map(conn: &Connection, id: i64) -> Vec<(String, String)> {
    let mut stmt = match conn
        .prepare(sql!("rankmath__select_post_taxonomies"))
    {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    // Best-effort: primary-term candidates render an empty select rather
    // than fail the editor — row decode errors are skipped on purpose.
    let taxs: Vec<String> = stmt
        .query_map([id], |r| r.get(0))
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default();
    let mut out = Vec::new();
    for t in taxs {
        // candidate suffixes: canonical label first — when both
        // `rank_math_primary_Région` and `rank_math_primary_region` exist,
        // the label spelling is what WP wrote last.
        let mut cands = vec![primary_key_default(&t), t.clone(), t.to_lowercase()];
        cands.dedup();
        let mut suffix = primary_key_default(&t);
        for c in &cands {
            let key = format!("rank_math_primary_{c}");
            let exists: bool = conn
                .query_row(
                    sql!("rankmath__select_post_meta_exists"),
                    rusqlite::params![id, key],
                    |_| Ok(()),
                )
                .is_ok();
            if exists {
                suffix = c.clone();
                break;
            }
        }
        out.push((t, suffix));
    }
    out
}

/// All `rank_math_*` metas for the post: `key → first value`.
fn rankmath_meta(conn: &Connection, id: i64) -> HashMap<String, String> {
    let mut m = HashMap::new();
    if let Ok(mut s) = conn.prepare(
        sql!("rankmath__select_meta"),
    ) {
        if let Ok(rows) = s.query_map([id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))) {
            // Meta-box helpers degrade to empty rather than 500; a skipped
            // row just omits one rank_math key from the form.
            for kv in rows.flatten() {
                m.entry(kv.0).or_insert(kv.1);
            }
        }
    }
    m
}

/// Post's own terms grouped by taxonomy for the primary-term selects:
/// `taxonomy → [(term_slug, name)]` (BTreeMap keeps taxonomy order stable).
fn post_terms(conn: &Connection, id: i64) -> BTreeMap<String, Vec<(String, String)>> {
    let mut out: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    if let Ok(mut s) = conn.prepare(
        sql!("rankmath__select_post_terms"),
    ) {
        if let Ok(rows) = s.query_map([id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        }) {
            // Same best-effort contract as rankmath_meta above.
            for (t, slug, name) in rows.flatten() {
                out.entry(t).or_default().push((slug, name));
            }
        }
    }
    out
}

fn row(label: &str, inner: &str) -> String {
    format!(
        "<div class=\"rm-row\"><label class=\"rm-label\">{label}</label><div class=\"rm-field\">{inner}</div></div>"
    )
}

fn text_field(key: &str, value: &str) -> String {
    format!(
        "<input type=\"text\" class=\"rm-in\" data-rm=\"{key}\" value=\"{}\" style=\"width:100%\">",
        esc(value)
    )
}

/// Metabox HTML. `f.id = None` → empty metas (new post).
pub(crate) fn metabox_html(sc: &SiteCtx, s: &Sess, f: &crud::PostForm) -> String {
    let meta: HashMap<String, String> = f
        .id
        .and_then(|id| crate::db::conn(&sc.db).ok().map(|c| (id, c)))
        .map(|(id, c)| rankmath_meta(&c, id))
        .unwrap_or_default();
    let terms: BTreeMap<String, Vec<(String, String)>> = f
        .id
        .and_then(|id| crate::db::conn(&sc.db).ok().map(|c| (id, c)))
        .map(|(id, c)| post_terms(&c, id))
        .unwrap_or_default();
    let tmap: Vec<(String, String)> = f
        .id
        .and_then(|id| crate::db::conn(&sc.db).ok().map(|c| (id, c)))
        .map(|(id, c)| tax_map(&c, id))
        .unwrap_or_default();

    let m = |k: &str| meta.get(k).cloned().unwrap_or_default();
    // Analysis accordion renders async — `seo_analysis` includes a
    // full-table inbound-links scan, so it must stay off the edit-page
    // render path (same reasoning as the list cell). Score pill still
    // falls back to the stored `rank_math_seo_score` meta.
    let ana: Option<crud::SeoAnalysis> = None;
    let score = ana.as_ref().map(|a| a.score).unwrap_or_else(|| m("rank_math_seo_score").parse().unwrap_or(0));
    let score_cls = if score >= 80 { "rm-good" } else if score >= 50 { "rm-ok" } else { "rm-bad" };

    // robots: serialized PHP array → checked boxes
    let robots = crud::robots_decode(&m("rank_math_robots"));
    let has = |v: &str| robots.iter().any(|x| x == v);
    // WP/Rank Math treats "default" as index+follow when only index stored
    let cb = |val: &str, label: &str, checked: bool| {
        format!(
            "<label class=\"rm-cb\"><input type=\"checkbox\" class=\"rm-robots\" value=\"{val}\"{}> {label}</label>",
            if checked { " checked" } else { "" }
        )
    };
    let robots_html = format!(
        "{}{}{}{}{}{}{}",
        cb("index", "Index", has("index")),
        cb("noindex", "No Index", has("noindex")),
        cb("follow", "Follow", has("follow")),
        cb("nofollow", "No Follow", has("nofollow")),
        cb("noarchive", "No Archive", has("noarchive")),
        cb("nosnippet", "No Snippet", has("nosnippet")),
        cb("noimageindex", "No Image Index", has("noimageindex")),
    );

    // schema type
    let schema_type = ["Article", "BlogPosting", "VideoObject"]
        .iter()
        .find(|t| !m(&format!("rank_math_schema_{t}")).is_empty())
        .copied()
        .unwrap_or("");
    let schema_radio = |t: &str| {
        format!(
            "<label class=\"rm-cb\"><input type=\"radio\" name=\"rm_schema\" value=\"{t}\"{}> {t}</label>",
            if schema_type == t { " checked" } else { "" }
        )
    };

    // primary selects — one per taxonomy the post actually uses
    let mut primary_html = String::new();
    for (tax, terms) in &terms {
        if tax == "post_tag" {
            continue; // Rank Math doesn't primary-tag
        }
        let suffix = tmap
            .iter()
            .find(|(t, _)| t == tax)
            .map(|(_, x)| x.clone())
            .unwrap_or_else(|| primary_key_default(tax));
        let cur = m(&format!("rank_math_primary_{suffix}"));
        let label = tax_label(tax).1;
        let mut sel = format!(
            "<select class=\"rm-in rm-primary\" data-tax=\"{tax}\" style=\"width:100%\"><option value=\"\">—</option>"
        );
        for (slug, name) in terms {
            sel += &format!(
                "<option value=\"{slug}\"{}>{}</option>",
                if *slug == cur { " selected" } else { "" },
                esc(name)
            );
        }
        sel += "</select>";
        primary_html += &row(&format!("Primary {label}"), &sel);
    }

    let resolved_title = crud::resolve_seo(
        &if m("rank_math_title").is_empty() {
            "%title% %sep% %sitename%".to_string()
        } else {
            m("rank_math_title")
        },
        &f.title,
        &crud::site_name(&sc.db),
    );
    let permalink = if f.id.is_some() {
        crate::admin::posts::public_path(sc, &f.lang, &f.kind, &f.slug)
    } else {
        String::new()
    };

    // ---- analysis accordion (lazy — fetched from /admin/seo/analysis/{id}) ----
    let analysis_html = match f.id {
        Some(id) => format!(
            "<div class=\"rm-analysis\" id=\"rm-ana\" data-id=\"{id}\"><div class=\"rm-summary\">Running analysis…</div></div>\
             <script>fetch('/admin/seo/analysis/{id}').then(function(r){{return r.text()}}).then(function(h){{var el=document.getElementById('rm-ana');if(el)el.innerHTML=h}})</script>"
        ),
        None => "<div class=\"rm-analysis\"><div class=\"rm-summary\">Save the post to run the analysis.</div></div>".into(),
    };
    let chips: String = m("rank_math_focus_keyword")
        .split(',')
        .map(|x| x.trim())
        .filter(|x| !x.is_empty())
        .enumerate()
        .map(|(i, k)| {
            format!(
                "<span class=\"rm-chip{}\"><span class=\"rm-x\">×</span>{} {}</span>",
                if i == 0 { " first" } else { "" },
                if i == 0 { "★" } else { "" },
                esc(k)
            )
        })
        .collect();
    let kw_input = format!(
        "<div class=\"rm-chips\" id=\"rm-chips\">{chips}<input type=\"text\" id=\"rm-kw-input\" placeholder=\"Example: Rank Math SEO\"></div><input type=\"hidden\" data-rm=\"rank_math_focus_keyword\" id=\"rm-kw-field\" value=\"{}\">",
        esc(&m("rank_math_focus_keyword"))
    );
    let pillar = m("rank_math_pillar_content") == "on";

    let html = format!(
        r##"<div class="postbox" id="rank-math-metabox"><h2>Rank Math SEO <span id="rm-score" class="rm-pill {score_cls}">{score}/100</span></h2><div class="inside">
<style>
#rank-math-metabox .inside{{padding:0 12px 12px}}
#rank-math-metabox .rm-tabs{{border-bottom:1px solid #dcdcde;margin:0 -12px 14px;padding:0 12px}}
#rank-math-metabox .rm-tab{{display:inline-block;padding:8px 10px 7px;cursor:pointer;color:#444;font-size:13px;border-bottom:2px solid transparent}}
#rank-math-metabox .rm-tab.on{{color:#000;font-weight:600;border-bottom-color:#2271b1}}
#rank-math-metabox .rm-pane{{display:none}}
#rank-math-metabox .rm-pane.on{{display:block}}
#rank-math-metabox .rm-sec{{font-size:13px;font-weight:600;color:#1d2327;margin:14px 0 6px}}
#rank-math-metabox .rm-prev{{padding:0;background:#fff}}
#rank-math-metabox .rm-prev-url{{color:#202124;font-size:12px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}}
#rank-math-metabox .rm-prev-title{{color:#1a0dab;font-size:16px;line-height:1.3;margin:2px 0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}}
#rank-math-metabox .rm-prev-desc{{color:#4d5156;font-size:12px;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden}}
#rank-math-metabox .rm-edit-snippet{{margin:10px 0 4px;background:#2271b1;color:#fff;border:none;padding:7px 14px;border-radius:4px;cursor:pointer;font-size:13px}}
#rank-math-metabox .rm-snippet{{display:none}}
#rank-math-metabox .rm-snippet.on{{display:block}}
#rank-math-metabox .rm-field input[type=text],#rank-math-metabox textarea.rm-in{{width:100%;box-sizing:border-box}}
#rank-math-metabox .rm-var{{display:inline-block;margin:0 4px 4px 0;padding:3px 10px;border:1px solid #2271b1;color:#2271b1;background:#fff;border-radius:3px;cursor:pointer;font-size:12px}}
#rank-math-metabox .rm-count{{font-size:12px;color:#646970}}
#rank-math-metabox .rm-count.over{{color:#dc3232}}
#rank-math-metabox .rm-chips{{border:1px solid #dcdcde;padding:6px;background:#fff;min-height:34px;cursor:text}}
#rank-math-metabox .rm-chip{{display:inline-block;background:#7ac943;color:#fff;border-radius:12px;padding:2px 10px;margin:2px 4px 2px 0;font-size:12px}}
#rank-math-metabox .rm-chip.first{{background:#46b450}}
#rank-math-metabox .rm-x{{cursor:pointer;margin-right:5px;font-weight:700}}
#rank-math-metabox #rm-kw-input{{border:0;outline:0;min-width:130px;box-shadow:none;padding:3px;width:90%}}
#rank-math-metabox .rm-cb{{display:block;margin:10px 0;font-weight:400}}
#rank-math-metabox .rm-pill{{font-size:12px;padding:1px 8px;border-radius:10px;color:#fff;vertical-align:middle}}
#rank-math-metabox .rm-good{{background:#46b450}}#rank-math-metabox .rm-ok{{background:#ffb900}}#rank-math-metabox .rm-bad{{background:#dc3232}}
#rank-math-metabox .rm-group{{border-bottom:1px solid #e2e2e2;margin-top:6px}}
#rank-math-metabox .rm-ghead{{padding:9px 0;cursor:pointer;font-weight:600;font-size:14px;display:flex;align-items:center;gap:8px}}
#rank-math-metabox .rm-arrow{{margin-left:auto;color:#787c82;font-weight:400}}
#rank-math-metabox .rm-glist{{display:none;margin:0 0 10px 0;padding:0;list-style:none}}
#rank-math-metabox .rm-group.on .rm-glist{{display:block}}
#rank-math-metabox .rm-glist li{{padding:3px 0 3px 2px;font-size:13px;color:#1d2327}}
#rank-math-metabox .rm-glist li .rm-ic{{display:inline-block;width:18px;font-weight:700}}
#rank-math-metabox .rm-pass .rm-ic{{color:#46b450}}
#rank-math-metabox .rm-fail .rm-ic{{color:#dc3232}}
#rank-math-metabox .rm-gpill{{font-size:12px;padding:2px 9px;border-radius:10px;color:#fff;font-weight:400}}
#rank-math-metabox .rm-gpill.good{{background:#46b450}}
#rank-math-metabox .rm-gpill.bad{{background:#e25959}}
#rank-math-metabox .rm-summary{{padding:8px 0;color:#646970;font-size:12px}}
#rank-math-metabox .rm-modal{{display:none;position:fixed;inset:0;background:rgba(0,0,0,.45);z-index:100000}}
#rank-math-metabox .rm-modal.on{{display:block}}
#rank-math-metabox .rm-modal-card{{position:absolute;left:50%;top:60px;transform:translateX(-50%);width:660px;max-width:92vw;background:#fff;border-radius:8px;box-shadow:0 8px 30px rgba(0,0,0,.25)}}
#rank-math-metabox .rm-modal-head{{display:flex;align-items:center;justify-content:space-between;padding:14px 18px;border-bottom:1px solid #e2e2e2;font-weight:600}}
#rank-math-metabox .rm-modal-x{{background:none;border:0;font-size:20px;cursor:pointer;color:#787c82}}
#rank-math-metabox .rm-modal-tabs{{border-bottom:1px solid #e2e2e2;padding:0 18px}}
#rank-math-metabox .rm-modal-tabs span{{display:inline-block;padding:10px 14px;font-size:13px;color:#444;cursor:pointer}}
#rank-math-metabox .rm-modal-tabs span.on{{color:#000;font-weight:600;border-bottom:2px solid #2271b1;margin-bottom:-1px}}
#rank-math-metabox .rm-modal-body{{padding:16px 18px 22px;max-height:70vh;overflow:auto}}
#rank-math-metabox .rm-modal-sec{{font-weight:600;font-size:14px;margin-bottom:8px}}
#rank-math-metabox .rm-modal-prev{{margin-bottom:16px}}
#rank-math-metabox .rm-fld{{background:#f6f7f7;border-radius:6px;padding:12px;margin-bottom:12px}}
#rank-math-metabox .rm-fld-head{{display:flex;align-items:center;gap:10px;margin-bottom:8px}}
#rank-math-metabox .rm-fld-label{{font-weight:600;font-size:13px}}
#rank-math-metabox .rm-fld-count{{color:#787c82;font-size:12px;margin-left:auto}}
#rank-math-metabox .rm-bar{{width:110px;height:6px;background:#e0e0e0;border-radius:3px;overflow:hidden}}
#rank-math-metabox .rm-bar i{{display:block;height:100%;background:#e25959;width:0}}
#rank-math-metabox .rm-modal-in{{width:100%;box-sizing:border-box}}
#rank-math-metabox .rm-fld-note{{color:#787c82;font-size:12px;margin-top:6px}}
</style>
<div class="rm-tabs">
 <span class="rm-tab on" data-p="general">General</span>
 <span class="rm-tab" data-p="advanced">Advanced</span>
 <span class="rm-tab" data-p="schema">Schema</span>
 <span class="rm-tab" data-p="social">Social</span>
</div>
<div class="rm-pane on" data-p="general">
 <div class="rm-sec">Preview</div>
 <div class="rm-prev">
  <div class="rm-prev-url" id="rm-pv-url">{pv_url}</div>
  <div class="rm-prev-title" id="rm-pv-title">{pv_title}</div>
  <div class="rm-prev-desc" id="rm-pv-desc">{pv_desc}</div>
 </div>
 <button type="button" class="rm-edit-snippet" id="rm-edit-snippet">Edit Snippet</button>

<div class="rm-modal" id="rm-modal">
 <div class="rm-modal-card">
  <div class="rm-modal-head"><span>Preview Snippet Editor</span><button type="button" class="rm-modal-x" id="rm-modal-x">×</button></div>
  <div class="rm-modal-tabs"><span class="on">⚙ General</span><span>⤳ Social</span><span>♥</span></div>
  <div class="rm-modal-body">
   <div class="rm-modal-sec">Preview</div>
   <div class="rm-prev rm-modal-prev">
    <div class="rm-prev-url" id="rm-m-pv-url">{pv_url}</div>
    <div class="rm-prev-title" id="rm-m-pv-title">{pv_title}</div>
    <div class="rm-prev-desc" id="rm-m-pv-desc">{pv_desc}</div>
   </div>
   <div class="rm-fld">
    <div class="rm-fld-head"><span class="rm-fld-label">Title</span><span class="rm-fld-count" id="rm-ct-title"></span><span class="rm-bar" id="rm-bar-title"><i></i></span></div>
    <input type="text" class="rm-modal-in" data-rm="rank_math_title" id="rm-f-title" value="{title_val}">
    <div class="rm-fld-note">This is what will appear in the first line when this post shows up in the search results.</div>
   </div>
   <div class="rm-fld">
    <div class="rm-fld-head"><span class="rm-fld-label">Permalink</span><span class="rm-fld-count" id="rm-ct-slug"></span><span class="rm-bar" id="rm-bar-slug"><i></i></span></div>
    <input type="text" class="rm-modal-in" id="rm-f-slug">
    <div class="rm-fld-note">This is the unique URL of this page, displayed below the post title in the search results.</div>
   </div>
   <div class="rm-fld">
    <div class="rm-fld-head"><span class="rm-fld-label">Description</span><span class="rm-fld-count" id="rm-ct-desc"></span><span class="rm-bar" id="rm-bar-desc"><i></i></span></div>
    <textarea class="rm-modal-in" data-rm="rank_math_description" id="rm-f-desc" rows="4">{desc_val}</textarea>
    <div class="rm-fld-note">This is what will appear as the description when this post shows up in the search results.</div>
   </div>
  </div>
 </div>
</div>
 <div class="rm-sec">Focus Keyword</div>
 {kw_input}
 <label class="rm-cb"><input type="checkbox" data-rm="rank_math_pillar_content"{pillar_chk}> This post is Pillar Content</label>
 {analysis_html}
</div>
<div class="rm-pane" data-p="advanced">
 {robots_row}
 {canonical_row}
 {primary_html}
</div>
<div class="rm-pane" data-p="schema">
 {snippet_row}
 {schema_row}
</div>
<div class="rm-pane" data-p="social">
 {fb_title}
 {fb_desc}
 {fb_img}
 {tw_card}
 {tw_title}
 {tw_desc}
 {tw_img}
</div>
<input type="hidden" name="rank_math" id="rank_math" value="">
</div></div>
<script>
(function(){{
 var box=document.getElementById('rank-math-metabox');
 if(!box)return;
 var postTitle=(document.getElementById('title')||{{}}).value||'';
 var sep=' - ',sitename={sitename_json};
 var vars={{'%title%':postTitle,'%sep%':'-','%sitename%':sitename}};
 function res(t){{for(var k in vars)t=t.split(k).join(vars[k]);return t}}
 function g(k){{var e=box.querySelector('[data-rm="'+k+'"]');return e?e.value:''}}
 function prev(){{
  var t=g('rank_math_title')||'%title% %sep% %sitename%';
  var d=g('rank_math_description');
  document.getElementById('rm-pv-title').textContent=res(t);
  document.getElementById('rm-pv-desc').textContent=d||{pv_desc_js};
 }}
 function count(){{var e=box.querySelector('[data-rm="rank_math_description"]');var c=document.getElementById('rm-desc-count');
  if(e&&c){{c.textContent=e.value.length+'/160';c.className='rm-count'+(e.value.length>160?' over':'')}}
 }}
 box.querySelectorAll('.rm-tab').forEach(function(t){{
  t.addEventListener('click',function(){{
   box.querySelectorAll('.rm-tab').forEach(function(x){{x.classList.remove('on')}});
   box.querySelectorAll('.rm-pane').forEach(function(x){{x.classList.remove('on')}});
   t.classList.add('on');box.querySelector('.rm-pane[data-p="'+t.dataset.p+'"]').classList.add('on');
  }});
 }});
 box.addEventListener('input',function(e){{if(e.target.dataset&&e.target.dataset.rm){{prev();count()}}}});
 // analysis accordions
 box.querySelectorAll('.rm-ghead').forEach(function(h){{
  h.addEventListener('click',function(){{
   var g=h.closest('.rm-group');g.classList.toggle('on');
   h.querySelector('.rm-arrow').textContent=g.classList.contains('on')?'⌃':'⌄';
  }});
 }});
 // keyword chips
 var kwField=document.getElementById('rm-kw-field'),kwInput=document.getElementById('rm-kw-input'),chipBox=document.getElementById('rm-chips');
 function kwSync(){{
  var kws=kwField.value.split(',').map(function(s){{return s.trim()}}).filter(Boolean);
  chipBox.querySelectorAll('.rm-chip').forEach(function(c){{c.remove()}});
  kws.forEach(function(k,i){{
   var s=document.createElement('span');s.className='rm-chip'+(i===0?' first':'');
   s.innerHTML='<span class="rm-x">×</span>'+(i===0?'★ ':'')+k.replace(/</g,'&lt;');
   s.querySelector('.rm-x').addEventListener('click',function(){{
    kws.splice(i,1);kwField.value=kws.join(',');kwSync();kwField.dispatchEvent(new Event('input',{{bubbles:true}}));
   }});
   chipBox.insertBefore(s,kwInput);
  }});
 }}
 if(kwInput){{
  kwInput.addEventListener('keydown',function(e){{
   if(e.key==='Enter'||e.key===','){{e.preventDefault();
    var k=kwInput.value.trim().replace(/,/g,'');
    if(k){{var kws=kwField.value.split(',').map(function(s){{return s.trim()}}).filter(Boolean);kws.push(k);kwField.value=kws.join(',');kwSync()}}
    kwInput.value='';}}
  }});
  chipBox.addEventListener('click',function(){{kwInput.focus()}});
 }}
 // snippet editor modal
 var modal=document.getElementById('rm-modal'),esb=document.getElementById('rm-edit-snippet');
 var fTitle=document.getElementById('rm-f-title'),fSlug=document.getElementById('rm-f-slug'),fDesc=document.getElementById('rm-f-desc');
 function bar(id,frac){{var b=document.getElementById('rm-bar-'+id);if(!b)return;
  var i=b.querySelector('i');var f=Math.max(0,Math.min(1,frac));i.style.width=(f*100)+'%';
  i.style.background=(f>1||(f>=1))?'#e25959':(f>0.85?'#8dc153':(f>0.5?'#f0b849':'#e25959'));}}
 function px(el,s){{var c=document.createElement('canvas');var x=c.getContext('2d');
  x.font=getComputedStyle(el).fontWeight+' '+getComputedStyle(el).fontSize+' '+getComputedStyle(el).fontFamily;
  return x.measureText(s).width;}}
 function counts(){{
  var t=(fTitle?fTitle.value:'').trim()||'%title% %sep% %sitename%';
  var d=(fDesc?fDesc.value:'');
  var s=fSlug?fSlug.value:'';
  var tpx=px(document.getElementById('rm-m-pv-title')||fTitle,res(t));
  document.getElementById('rm-ct-title').textContent=res(t).length+' / 60 ('+Math.round(tpx)+'px / 580px)';
  bar('title',tpx/580);
  document.getElementById('rm-ct-desc').textContent=d.length+' / 160 ('+Math.round(px(fDesc||fTitle,d))+'px / 920px)';
  bar('desc',px(fDesc||fTitle,d)/920);
  document.getElementById('rm-ct-slug').textContent=s.length+' / 75';
  bar('slug',s.length/75);
 }}
 function mprev(){{
  var t=(fTitle?fTitle.value:'')||'%title% %sep% %sitename%';
  document.getElementById('rm-m-pv-title').textContent=res(t);
  var d=fDesc?fDesc.value:'';document.getElementById('rm-m-pv-desc').textContent=d||{pv_desc_js};
  var s=fSlug?fSlug.value:'';if(s){{var u=document.getElementById('rm-m-pv-url');u.textContent=u.textContent.replace(/[^\/]+\/$/,s+'/');}}
 }}
 if(esb)esb.addEventListener('click',function(){{
  var slugEl=document.getElementById('post_name');
  if(fSlug&&slugEl)fSlug.value=slugEl.value;
  modal.classList.add('on');mprev();counts();
 }});
 var mx=document.getElementById('rm-modal-x');
 if(mx)mx.addEventListener('click',function(){{modal.classList.remove('on');prev();count();}});
 if(modal)modal.addEventListener('click',function(e){{if(e.target===modal){{modal.classList.remove('on');prev();count();}}}});
 [fTitle,fSlug,fDesc].forEach(function(el){{if(el)el.addEventListener('input',function(){{
  if(el===fSlug){{var sl=document.getElementById('post_name');if(sl)sl.value=el.value}}
  mprev();counts();
 }})}});
 prev();count();
 var form=box.closest('form');
 if(form)form.addEventListener('submit',function(){{
  var o={{}};
  box.querySelectorAll('[data-rm]').forEach(function(e){{
   if(e.type==='checkbox'){{o[e.dataset.rm]=e.checked?'on':''}}
   else{{o[e.dataset.rm]=e.value}}
  }});
  o.rank_math_robots=Array.from(box.querySelectorAll('.rm-robots:checked')).map(function(e){{return e.value}});
  var sc=box.querySelector('input[name=rm_schema]:checked');
  o.rank_math_schema_type=sc?sc.value:'';
  var sn=box.querySelector('[data-rm="rank_math_rich_snippet"]');
  o.rank_math_rich_snippet=(sn&&sn.checked)?'on':'';
  var pl=box.querySelector('[data-rm="rank_math_pillar_content"]');
  o.rank_math_pillar_content=(pl&&pl.checked)?'on':'';
  o.primary={{}};
  box.querySelectorAll('.rm-primary').forEach(function(e){{o.primary[e.dataset.tax]=e.value}});
  document.getElementById('rank_math').value=JSON.stringify(o);
 }});
}})();
</script>"##,
        kw_input = kw_input,
        pillar_chk = if pillar { " checked" } else { "" },
        analysis_html = analysis_html,
        title_val = esc(&m("rank_math_title")),
        desc_val = esc(&m("rank_math_description")),
        robots_row = row("Robots Meta", &robots_html),
        canonical_row = row("Canonical URL", &text_field("rank_math_canonical_url", &m("rank_math_canonical_url"))),
        snippet_row = row("Rich Snippet", &format!(
            "<label class=\"rm-cb\"><input type=\"checkbox\" data-rm=\"rank_math_rich_snippet\"{}> Enable</label>",
            if m("rank_math_rich_snippet") == "on" { " checked" } else { "" }
        )),
        schema_row = row("Schema Type", &format!(
            "{}{}{}<label class=\"rm-cb\"><input type=\"radio\" name=\"rm_schema\" value=\"\"{}> None</label>",
            schema_radio("Article"),
            schema_radio("BlogPosting"),
            schema_radio("VideoObject"),
            if schema_type.is_empty() { " checked" } else { "" }
        )),
        fb_title = row("Facebook Title", &text_field("rank_math_facebook_title", &m("rank_math_facebook_title"))),
        fb_desc = row("Facebook Description", &format!(
            "<textarea class=\"rm-in\" data-rm=\"rank_math_facebook_description\" rows=\"2\">{}</textarea>",
            esc(&m("rank_math_facebook_description"))
        )),
        fb_img = row("Facebook Image", &text_field("rank_math_facebook_image", &m("rank_math_facebook_image"))),
        tw_card = row("Twitter Card Type", &format!(
            "<select class=\"rm-in\" data-rm=\"rank_math_twitter_card_type\" style=\"width:100%\">{}</select>",
            ["summary", "summary_large_image"]
                .iter()
                .map(|t| format!(
                    "<option value=\"{t}\"{}>{t}</option>",
                    if m("rank_math_twitter_card_type") == *t || (m("rank_math_twitter_card_type").is_empty() && *t == "summary_large_image") { " selected" } else { "" }
                ))
                .collect::<String>()
        )),
        tw_title = row("Twitter Title", &text_field("rank_math_twitter_title", &m("rank_math_twitter_title"))),
        tw_desc = row("Twitter Description", &format!(
            "<textarea class=\"rm-in\" data-rm=\"rank_math_twitter_description\" rows=\"2\">{}</textarea>",
            esc(&m("rank_math_twitter_description"))
        )),
        tw_img = row("Twitter Image", &text_field("rank_math_twitter_image", &m("rank_math_twitter_image"))),
        pv_url = esc(&permalink),
        pv_title = esc(&resolved_title),
        pv_desc = esc(&m("rank_math_description")),
        pv_desc_js = serde_json::to_string(&m("rank_math_description")).unwrap_or_else(|_| "\"\"".into()),
        sitename_json = serde_json::to_string(&crud::site_name(&sc.db)).unwrap_or_else(|_| "\"\"".into()),
        score = score,
        score_cls = score_cls,
        primary_html = primary_html,
    );
    let _ = s;
    html
}
