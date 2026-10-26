//! WP-style Settings pages. Stores every value in `settings(key,value)`.
//! SMTP/pingback/Gravatar-dependent controls render disabled with an honest note.

use super::*;
use crate::db::comments as sdb;
use axum::extract::{Form, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use std::collections::HashMap;

fn require_admin(s: &Sess) -> Result<(), StatusCode> {
    if s.role == "admin" { Ok(()) } else { Err(StatusCode::FORBIDDEN) }
}

fn get(db: &std::path::Path, key: &str, default: &str) -> String {
    sdb::setting_get(db, key).unwrap_or_else(|| default.to_string())
}

fn row(label: &str, html: &str) -> String {
    format!("<tr><th scope=\"row\">{}</th><td>{}</td></tr>", esc(label), html)
}

fn checkbox(name: &str, checked: bool, label: &str) -> String {
    format!("<label><input type=\"checkbox\" name=\"{name}\" value=\"1\"{}> {}</label>", if checked { " checked" } else { "" }, esc(label))
}

fn text(name: &str, value: &str, class: &str) -> String {
    format!("<input name=\"{name}\" value=\"{}\" class=\"{class}\">", esc(value))
}

fn textarea(name: &str, value: &str, rows: usize) -> String {
    format!("<textarea name=\"{name}\" rows=\"{rows}\" class=\"large-text code\">{}</textarea>", esc(value))
}

fn page_shell(s: &Sess, section: &str, title: &str, inner: &str, updated: bool) -> Response {
    let notice = if updated { "<div class=\"notice notice-success\"><p>Settings saved.</p></div>" } else { "" };
    let body = format!(
        "{}<div class=\"wrap\"><h1>{}</h1>{notice}<form method=\"post\" action=\"/admin/settings/{}\"><input type=\"hidden\" name=\"csrf\" value=\"{}\"><table class=\"form-table\" role=\"presentation\"><tbody>{}</tbody></table><p class=\"submit\"><button class=\"button button-primary\">Save Changes</button></p></form></div>",
        chrome("settings", s), esc(title), section, esc(&s.csrf), inner
    );
    html_admin(s, title, &body, true, &format!("settings-{section}")).into_response()
}

/// `<select>` of published pages — Reading/Privacy need real dropdowns, not
/// raw IDs. `selected` is the stored post id (0 = none/— Select —).
fn pages_select(name: &str, selected: i64, sc: &SiteCtx, none_label: &str) -> String {
    let mut opts = format!("<option value=\"0\">{}</option>", esc(none_label));
    for (id, title) in crud::menus::panel_posts(&sc.db, "page", "", false).unwrap_or_default() {
        let sel = if id == selected { " selected" } else { "" };
        let t = if title.is_empty() { "(no title)".to_string() } else { esc(&title) };
        opts.push_str(&format!("<option value=\"{id}\"{sel}>{t}</option>"));
    }
    format!("<select name=\"{name}\" id=\"{name}\">{opts}</select>")
}

fn radio_row(options: &[(&str, &str)], name: &str, current: &str, enabled: &[&str]) -> String {
    let mut out = String::new();
    for (val, label) in options {
        let chk = if *val == current { " checked" } else { "" };
        let dis = if enabled.contains(val) { "" } else { " disabled" };
        out.push_str(&format!("<label><input type=\"radio\" name=\"{name}\" value=\"{val}\"{chk}{dis}> {label}</label><br>"));
    }
    out
}

fn select_opt(name: &str, options: &[(&str, &str)], current: &str, class: &str) -> String {
    let mut opts = String::new();
    for (val, label) in options {
        let sel = if *val == current { " selected" } else { "" };
        opts.push_str(&format!("<option value=\"{val}\"{sel}>{label}</option>"));
    }
    format!("<select name=\"{name}\" id=\"{name}\" class=\"{class}\">{opts}</select>")
}

fn general(sc: &SiteCtx) -> String {
    let roles = [("editor", "Editor"), ("admin", "Administrator")];
    let days = [("0", "Sunday"), ("1", "Monday"), ("2", "Tuesday"), ("3", "Wednesday"), ("4", "Thursday"), ("5", "Friday"), ("6", "Saturday")];
    let tzs = ["Europe/Paris", "Europe/Madrid", "Asia/Ho_Chi_Minh", "UTC", "America/New_York", "America/Los_Angeles"];
    let tz_cur = get(&sc.db, "timezone", "Europe/Paris");
    let mut tz_opts: Vec<(String, String)> = tzs.iter().map(|t| (t.to_string(), t.to_string())).collect();
    if !tzs.contains(&tz_cur.as_str()) { tz_opts.push((tz_cur.clone(), tz_cur.clone())); }
    let tz_pairs: Vec<(&str, &str)> = tz_opts.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();

    row("Site Title", &text("site_title", &get(&sc.db, "site_title", &sc.name), "regular-text"))
    + &row("Tagline", &text("tagline", &get(&sc.db, "tagline", ""), "regular-text"))
    + &row("WordPress Address (URL)", "<input class=\"regular-text code\" disabled value=\"\" title=\"One address per build — Site Address is the source\">")
    + &row("Site Address (URL)", &text("site_url", &get(&sc.db, "site_url", "/"), "regular-text code"))
    + &row("Asset Origin (URL)", &(text("asset_origin", &get(&sc.db, "asset_origin", ""), "regular-text code") + " <p class=\"description\">Static files not present locally (CSS/JS/uploads) redirect here — e.g. <code>https://test.example.com</code> while testing locally. Empty = same as Site Address.</p>"))
    + &row("Membership", "<label><input type=\"checkbox\" disabled> Anyone can register</label> <p class=\"description\">Public registration is not part of this build.</p>")
    + &row("New User Default Role", &select_opt("default_role", &roles, &get(&sc.db, "default_role", "editor"), ""))
    + &row("Site Language", "<p class=\"description\">Cosmetic — every post carries its own <code>lang</code> column; nothing reads this setting.</p>")
    + &row("Timezone", &select_opt("timezone", &tz_pairs, &tz_cur, ""))
    + &row("Date Format", &text("date_format", &get(&sc.db, "date_format", "F j, Y"), "regular-text"))
    + &row("Time Format", &text("time_format", &get(&sc.db, "time_format", "H:i"), "regular-text"))
    + &row("Week Starts On", &select_opt("start_of_week", &days, &get(&sc.db, "start_of_week", "1"), ""))
}

fn writing(sc: &SiteCtx) -> String {
    let mut cats = String::from("<option value=\"\">— Select —</option>");
    for t in crud::list_terms(&sc.db, "category").unwrap_or_default() {
        let sel = if t.id.to_string() == get(&sc.db, "default_category", "") { " selected" } else { "" };
        cats.push_str(&format!("<option value=\"{}\"{}>{}</option>", t.id, sel, esc(&t.name)));
    }
    row("Default Post Category", &format!("<select name=\"default_category\" id=\"default_category\">{cats}</select>"))
        + &row("Post via email", "<em>Not available — SMTP is not configured in this build.</em>")
}

fn reading(sc: &SiteCtx) -> String {
    let show = get(&sc.db, "show_on_front", "posts");
    let page_on_front: i64 = get(&sc.db, "page_on_front", "0").parse().unwrap_or(0);
    // `page_for_posts` stays stored but hidden: the static build emits no
    // posts-index page to point it at (decided — fixed structure on a static
    // site until a blog index is scheduled).
    let home = format!(
        "<fieldset><legend class=\"screen-reader-text\">Your homepage displays</legend>\
        <label><input type=\"radio\" name=\"show_on_front\" value=\"posts\"{}> Your latest posts</label><br>\
        <label><input type=\"radio\" name=\"show_on_front\" value=\"page\"{}> A static page <span class=\"screen-reader-text\">(select below)</span></label>\
        <ul><li><label for=\"page_on_front\">Homepage: {}</label></li></ul>\
        <p class=\"description\">Posts page: fixed structure on static site — no posts index is generated.</p></fieldset>",
        if show != "page" { " checked" } else { "" },
        if show == "page" { " checked" } else { "" },
        pages_select("page_on_front", page_on_front, sc, "— Select —")
    );
    row("Your homepage displays", &home)
        + &row("Blog pages show at most", &(text("posts_per_page", &get(&sc.db, "posts_per_page", "10"), "small-text") + " <label>posts</label>"))
        + &row("Syndication feeds show the most recent", &(text("posts_per_rss", &get(&sc.db, "posts_per_rss", "10"), "small-text") + " <label for=\"posts_per_rss\">items</label>"))
        + &row("For each post in a feed, include", &radio_row(&[("0", "Full text"), ("1", "Excerpt")], "rss_use_excerpt", &get(&sc.db, "rss_use_excerpt", "0"), &["0", "1"]))
}

fn discussion(sc: &SiteCtx) -> String {
    row("Default article settings",
        &format!("{}<br>{}<br>{}",
            "<label><input type=\"checkbox\" disabled title=\"No pingback schema\"> Attempt to notify any blogs linked to from the post</label>",
            "<label><input type=\"checkbox\" disabled title=\"No pingback schema\"> Allow link notifications from other blogs (pingbacks and trackbacks) on new posts</label>",
            checkbox("default_comment_status", get(&sc.db, "default_comment_status", "open") == "open", "Allow people to submit comments on new posts")))
        + &row("Other comment settings", &format!("{}<br>{}<br>{}<br>{}<br>{}",
            checkbox("require_name_email", get(&sc.db, "require_name_email", "1") == "1", "Comment author must fill out name and email"),
            "<label><input type=\"checkbox\" disabled title=\"No public accounts\"> Users must be registered and logged in to comment</label>",
            checkbox("close_comments", get(&sc.db, "close_comments", "0") == "1", "Automatically close comments on posts older than days"),
            checkbox("comment_cookies", get(&sc.db, "comment_cookies", "1") == "1", "Show comments cookies opt-in checkbox"),
            checkbox("thread_comments", get(&sc.db, "thread_comments", "1") == "1", "Enable threaded (nested) comments levels deep")))
        + &row("Email me whenever", &format!("{}<br>{}<p class=\"description\">Not supported — no mail transport in this build.</p>",
            "<label><input type=\"checkbox\" disabled title=\"Not supported — no mail transport\"> Anyone posts a comment</label>",
            "<label><input type=\"checkbox\" disabled title=\"Not supported — no mail transport\"> A comment is held for moderation</label>"))
        + &row("Before a comment appears", &format!("{}<br>{}",
            checkbox("comment_moderation", get(&sc.db, "comment_moderation", "0") == "1", "Comment must be manually approved"),
            checkbox("comment_previously_approved", get(&sc.db, "comment_previously_approved", "0") == "1", "Comment author must have a previously approved comment")))
        + &row("Comment Moderation", &(text("comment_max_links", &get(&sc.db, "comment_max_links", "2"), "small-text") + " <label>links or more</label><br>" + &textarea("moderation_keys", &get(&sc.db, "moderation_keys", ""), 6)))
        + &row("Disallowed Comment Keys", &textarea("disallowed_keys", &get(&sc.db, "disallowed_keys", ""), 6))
        + &row("Avatars", "<label><input type=\"checkbox\" disabled> Show Avatars</label> <p class=\"description\">Disabled — no Gravatar integration.</p>")
}

fn media(sc: &SiteCtx) -> String {
    row("Thumbnail size", &(text("thumbnail_size_w", &get(&sc.db, "thumbnail_size_w", "150"), "small-text") + " × " + &text("thumbnail_size_h", &get(&sc.db, "thumbnail_size_h", "150"), "small-text") + " " + &checkbox("thumbnail_crop", get(&sc.db, "thumbnail_crop", "1") == "1", "Crop thumbnail to exact dimensions (normally thumbnails are proportional)")))
        + &row("Medium size", &(text("medium_size_w", &get(&sc.db, "medium_size_w", "300"), "small-text") + " × " + &text("medium_size_h", &get(&sc.db, "medium_size_h", "300"), "small-text")))
        + &row("Large size", &(text("large_size_w", &get(&sc.db, "large_size_w", "1024"), "small-text") + " × " + &text("large_size_h", &get(&sc.db, "large_size_h", "1024"), "small-text")))
        + &row("Uploading Files", &checkbox("uploads_use_yearmonth", get(&sc.db, "uploads_use_yearmonth", "1") == "1", "Organize my uploads into month- and year-based folders"))
}

fn permalink(sc: &SiteCtx) -> String {
    let cur = get(&sc.db, "permalink_structure", "/%postname%/");
    // Decided: the URL structure is fixed on a static site — changing it
    // would break SEO/redirects, so every radio/input renders disabled and
    // the stored values are shown read-only (values remain stored; generate
    // keeps the hardcoded structure either way).
    let radios = [
        ("plain", "Plain <code>?p=123</code>"),
        ("day", "Day and name <code>/%year%/%monthnum%/%day%/%postname%/</code>"),
        ("month", "Month and name <code>/%year%/%monthnum%/%postname%/</code>"),
        ("numeric", "Numeric <code>/archives/%post_id%</code>"),
        ("postname", "Post name <code>/%postname%/</code>"),
        ("custom", "Custom Structure"),
    ];
    let mut out = String::new();
    for (val, label) in radios {
        let chk = if cur.contains("%postname%") && val == "postname" { " checked" } else { "" };
        out.push_str(&format!("<label><input type=\"radio\" name=\"permalink_structure\" value=\"{val}\"{chk} disabled> {label}</label><br>"));
    }
    out.push_str(&format!("<input name=\"permalink_custom\" class=\"regular-text code\" value=\"{}\" disabled>", esc(&cur)));
    out.push_str("<p class=\"description\">Fixed structure on static site — permalinks are emitted by the generator and cannot be changed without breaking redirects.</p>");
    let base_note = |name: &str, def: &str| format!(
        "<input name=\"{name}\" value=\"{}\" class=\"regular-text\" disabled> <p class=\"description\">Fixed structure on static site.</p>",
        esc(&get(&sc.db, name, def))
    );
    row("Common Settings", &out)
        + &row("Category base", &base_note("category_base", "category"))
        + &row("Tag base", &base_note("tag_base", "tag"))
}

fn privacy(sc: &SiteCtx) -> String {
    let cur: i64 = get(&sc.db, "privacy_policy_page", "0").parse().unwrap_or(0);
    row("Privacy Policy page", &(pages_select("privacy_policy_page", cur, sc, "— Select —")
        + " <p class=\"description\">Pick the published page your theme uses for the privacy link; the setting is saved even if the current templates do not link it.</p>"))
}

pub(crate) async fn settings_page(State(app): State<App>, headers: HeaderMap, Path(section): Path<String>, axum::extract::Query(q): axum::extract::Query<HashMap<String, String>>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else { return Redirect::to("/admin/login").into_response(); };
    if let Err(e) = require_admin(&s) { return e.into_response(); }
    let (title, body) = match section.as_str() {
        "general" => ("General Settings", general(sc)),
        "writing" => ("Writing Settings", writing(sc)),
        "reading" => ("Reading Settings", reading(sc)),
        "discussion" => ("Discussion Settings", discussion(sc)),
        "media" => ("Media Settings", media(sc)),
        "permalink" => ("Permalink Settings", permalink(sc)),
        "privacy" => ("Privacy Settings", privacy(sc)),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    page_shell(&s, &section, title, &body, q.get("updated").is_some())
}

pub(crate) async fn settings_save(State(app): State<App>, headers: HeaderMap, Path(section): Path<String>, Form(f): Form<HashMap<String, String>>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else { return Redirect::to("/admin/login").into_response(); };
    if let Err(e) = require_admin(&s) { return e.into_response(); }
    if f.get("csrf").map(|v| v.as_str()) != Some(s.csrf.as_str()) { return StatusCode::FORBIDDEN.into_response(); }
    // Radio/checkbox normalization: checkboxes collapse to 1/0; permalink
    // radios keep only the stored structure string.
    for (k, present) in [
        ("default_comment_status", "open"),
        ("comment_moderation", "1"),
        ("comment_previously_approved", "1"),
        ("require_name_email", "1"),
        ("close_comments", "1"),
        ("comment_cookies", "1"),
        ("thread_comments", "1"),
        ("thumbnail_crop", "1"),
        ("uploads_use_yearmonth", "1"),
    ] {
        let v = if f.contains_key(k) { present } else if k == "default_comment_status" { "closed" } else { "0" };
        let _ = sdb::setting_set(&sc.db, k, v);
    }
    if section == "permalink" {
        // Permalink inputs render disabled (fixed structure on static
        // site), so nothing posts — never clobber the stored values with
        // defaults. A hand-crafted POST still lands through the generic
        // settings writer below; that's harmless since nothing reads them.
    }
    for (k, v) in &f {
        if k == "csrf" || k == "permalink_custom" || k == "permalink_structure"
            || matches!(k.as_str(), "default_comment_status"|"comment_moderation"|"comment_previously_approved"|"require_name_email"|"close_comments"|"comment_cookies"|"thread_comments"|"thumbnail_crop"|"uploads_use_yearmonth") { continue; }
        let _ = sdb::setting_set(&sc.db, k, v);
    }
    crate::admin::admin::bust_cache(&sc);
    Redirect::to(&format!("/admin/settings/{section}?updated=1")).into_response()
}
