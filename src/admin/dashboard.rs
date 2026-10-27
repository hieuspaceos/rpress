//! WP `index.php` dashboard parity as a real module (`mod dashboard;`).
//
// Widgets: At a Glance, Activity, Quick Draft, Site Health Status +
// Screen Options (per-widget toggles, 1–4 column layout) persisted as
// JSON in `settings.dashboard_{user_id}`.

use std::time::{SystemTime, UNIX_EPOCH};
#[allow(unused_imports)]
use crate::sql;
#[allow(unused_imports)]
use super::*;

/// WP widget ids — also the keys stored in the screen-options `hidden` list.
const WIDGETS: [(&str, &str); 4] = [
    ("dashboard_right_now", "At a Glance"),
    ("dashboard_activity", "Activity"),
    ("dashboard_quick_press", "Quick Draft"),
    ("dashboard_site_health", "Site Health Status"),
];

/// Per-user dashboard layout, persisted as `settings.dashboard_{user_id}`.
#[derive(serde::Serialize, serde::Deserialize)]
struct DashOpts {
    #[serde(default = "two")]
    columns: u8,
    #[serde(default)]
    hidden: Vec<String>,
}

fn two() -> u8 {
    2
}

fn dash_opts(sc: &SiteCtx, user_id: i64) -> DashOpts {
    crud::comments::setting_get(&sc.db, &format!("dashboard_{user_id}"))
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or(DashOpts {
            columns: 2,
            hidden: Vec::new(),
        })
}

/// `.postbox` wrapper — WP widget chrome (`hndle` header + `.inside` body).
pub(crate) fn postbox(id: &str, title: &str, inside: String) -> String {
    format!(
        "<div id=\"{id}\" class=\"postbox\"><h2 class=\"hndle\">{title}</h2><div class=\"inside\">{inside}</div></div>"
    )
}

/// WP Screen Options panel: per-widget hide toggles + column layout radios.
/// Toggled by `screenMeta` in common.min.js via `aria-controls`.
fn screen_options(opts: &DashOpts, csrf: &str) -> String {
    let mut checks = String::new();
    for (id, label) in WIDGETS {
        let checked = if opts.hidden.iter().any(|h| h == id) {
            ""
        } else {
            " checked"
        };
        checks.push_str(&format!(
            "<label for=\"{id}-hide\"><input type=\"checkbox\" id=\"{id}-hide\" name=\"{id}-hide\" value=\"1\"{checked}>{label}</label>"
        ));
    }
    let mut radios = String::new();
    for n in 1u8..=4 {
        let checked = if n == opts.columns { " checked" } else { "" };
        radios.push_str(&format!(
            "<label><input type=\"radio\" name=\"columns\" value=\"{n}\"{checked}> {n}</label> "
        ));
    }
    format!(
        r#"<div id="screen-meta" class="metabox-prefs">
<div id="screen-options-wrap" class="hidden" tabindex="-1" aria-label="Screen Options Tab">
<form method="post" action="/admin">
<input type="hidden" name="csrf" value="{csrf}">
<fieldset class="metabox-prefs"><legend>Screen elements</legend>{checks}</fieldset>
<fieldset class="metabox-prefs"><legend>Layout</legend>{radios}</fieldset>
<p class="submit"><input type="submit" class="button button-primary" value="Apply"></p>
</form>
</div>
<div id="screen-meta-links">
<div id="screen-options-link-wrap" class="hide-if-no-js screen-meta-toggle">
<button type="button" id="show-settings-link" class="button show-settings" aria-controls="screen-options-wrap" aria-expanded="false">Screen Options</button>
</div>
</div>
</div>"#
    )
}

/// At a Glance body: post/page/tour/media counts + comments/moderation +
/// honest version footer.
fn glance_box(c: &Connection) -> String {
    let n = |sql: &str| -> i64 {
        c.query_row(sql, [], |r| r.get(0)).unwrap_or(0)
    };
    // One card per registered kind (core built-ins + adapters) — a new
    // plugin kind appears automatically, no dashboard edit.
    let mut items = String::new();
    for def in plugins::kind_defs() {
        let n = c
            .query_row(
                "SELECT COUNT(*) FROM posts WHERE kind=?1 AND status='publish'",
                [def.kind],
                |r| r.get::<_, i64>(0),
            )
            .unwrap_or(0);
        let label = if n == 1 { def.noun.to_string() } else { format!("{}s", def.noun) };
        items.push_str(&format!(
            "<li><a href=\"/admin/{}\">{n} {}</a></li>",
            def.section, label
        ));
    }
    let media = n("SELECT COUNT(*) FROM media");
    let comments = n("SELECT COUNT(*) FROM comments WHERE status IN ('approved','pending')");
    let pending = n("SELECT COUNT(*) FROM comments WHERE status='pending'");
    postbox(
        "dashboard_right_now",
        "At a Glance",
        format!(
            r#"<div id="dashboard_right_now"><div class="main">
<ul>
{items}
<li><a href="/admin/media">{media} Media</a></li>
<li><a href="/admin/comments">{comments} Comments</a> · <a href="/admin/comments?status=pending">{pending} in moderation</a></li>
</ul>
</div>
<p id="wp-version-message"><span id="footer-version">rpress v{}</span></p>
</div>"#,
            env!("CARGO_PKG_VERSION")
        ),
    )
}

/// One Activity "Recently Published" row: linked title + date + Edit/Trash
/// row actions (trash posts to the existing `/admin/trash` handler).
fn activity_post_row(r: &crud::posts::DashPostRow, csrf: &str) -> String {
    let title = if r.title.trim().is_empty() {
        "(no title)".to_string()
    } else {
        esc(&r.title)
    };
    let date = r.date.get(..16).unwrap_or(&r.date).to_string();
    format!(
        r#"<li><span>{date}</span> <a href="/admin/edit/{id}">{title}</a>
<div class="row-actions"><span class="edit"><a href="/admin/edit/{id}">Edit</a></span> | <span class="trash"><button type="submit" class="button-link" form="dash-trash-{id}">Trash</button></span></div>
<form id="dash-trash-{id}" method="post" action="/admin/trash" style="display:none"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="id" value="{id}"><input type="hidden" name="status" value="trash"></form>
</li>"#,
        id = r.id,
    )
}

/// One Activity comment row: author + excerpt + "On: post" + inline
/// Approve/Spam/Trash posting to `comments::comment_action` (`back=/admin`).
fn activity_comment_row(r: &crud::comments::CommentRow, csrf: &str) -> String {
    let excerpt: String = r
        .body
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(80)
        .collect();
    let mut acts = String::new();
    if r.status == "pending" {
        acts.push_str(
            "<button type=\"submit\" class=\"button-link\" name=\"action\" value=\"approve\">Approve</button> · ",
        );
    }
    acts.push_str(
        "<button type=\"submit\" class=\"button-link\" name=\"action\" value=\"spam\">Spam</button> · <button type=\"submit\" class=\"button-link\" name=\"action\" value=\"trash\">Trash</button>",
    );
    format!(
        r#"<li class="comment"><form method="post" action="/admin/comments/action">
<input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="id" value="{id}"><input type="hidden" name="back" value="/admin">
<p><strong>{author}</strong> <em>{excerpt}</em>
<span class="dashboard-comment-wrap">— On: <a href="/admin/comments?post={pid}">{post}</a></span></p>
<p class="row-actions">{acts}</p>
</form></li>"#,
        id = r.id,
        author = esc(&r.author),
        excerpt = esc(&excerpt),
        pid = r.post_id.unwrap_or(0),
        post = esc(&r.post_title),
    )
}

/// Activity body: Awaiting Moderation (pending > 0), Recently Published,
/// Comments — WP `wp_dashboard_recent_posts` + `wp_dashboard_recent_comments`.
fn activity_box(_c: &Connection, sc: &SiteCtx, csrf: &str) -> String {
    let mut h = String::from("<div id=\"activity-widget\">");
    let pending = crate::db::pending_comments_count(&sc.db);
    if pending > 0 {
        let mut bubble = format!(
            "<div class=\"dashboard-pending-comments\"><span class=\"awaiting-mod count-{pending}\"><span class=\"pending-count\" aria-hidden=\"true\">{pending}</span></span> <a href=\"/admin/comments?status=pending\">pending moderation</a>"
        );
        if let Ok(Some(p)) = crud::comments::first_pending(&sc.db) {
            let excerpt: String = p
                .body
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(100)
                .collect();
            bubble.push_str(&format!(
                " — <strong>{}</strong>: {} <a href=\"/admin/comments?status=pending\">Moderate</a>",
                esc(&p.author),
                esc(&excerpt),
            ));
        }
        bubble.push_str("</div>");
        h.push_str(&bubble);
    }
    h.push_str("<div id=\"published-posts\" class=\"activity-block\"><h3>Recently Published</h3><ul>");
    match crud::posts::recent_posts(&sc.db, 5) {
        Ok(rows) if !rows.is_empty() => {
            for r in &rows {
                h.push_str(&activity_post_row(r, csrf));
            }
        }
        _ => h.push_str("<li>No recently published posts.</li>"),
    }
    h.push_str("</ul></div>");
    h.push_str("<div id=\"latest-comments\" class=\"activity-block\"><h3>Comments</h3><ul>");
    match crud::comments::recent_comments(&sc.db, 5) {
        Ok(rows) if !rows.is_empty() => {
            for r in &rows {
                h.push_str(&activity_comment_row(r, csrf));
            }
        }
        _ => h.push_str("<li>No comments yet.</li>"),
    }
    h.push_str("</ul><p class=\"textright\"><a href=\"/admin/comments\">View all</a></p></div></div>");
    postbox("dashboard_activity", "Activity", h)
}

/// Quick Draft body: title/content form → POST /admin/draft + the session
/// user's 3 most recent drafts (WP `wp_dashboard_quick_press`).
fn quick_draft_box(sc: &SiteCtx, s: &Sess) -> String {
    let mut h = format!(
        r#"<form method="post" action="/admin/draft" id="quick-press">
<input type="hidden" name="csrf" value="{}">
<div class="input-text-wrap" id="title-wrap"><label for="quick-draft-title">Title</label><input type="text" name="title" id="quick-draft-title" class="widefat" required></div>
<div class="textarea-wrap" id="description-wrap"><label for="quick-draft-content">Content</label><textarea name="content" id="quick-draft-content" class="widefat" rows="4" placeholder="What’s on your mind?"></textarea></div>
<p class="submit"><input type="submit" class="button button-primary" value="Save Draft"></p>
</form>"#,
        s.csrf
    );
    h.push_str("<h4>Your Recent Drafts</h4><ul>");
    match crud::posts::drafts(&sc.db, s.user_id, 3) {
        Ok(rows) if !rows.is_empty() => {
            for r in &rows {
                let title = if r.title.trim().is_empty() {
                    "(no title)".to_string()
                } else {
                    esc(&r.title)
                };
                let date = r.date.get(..16).unwrap_or(&r.date);
                h.push_str(&format!(
                    "<li><span>{date}</span> <a href=\"/admin/edit/{id}\">{title}</a> <a href=\"/admin/edit/{id}\" class=\"row-actions-link\">Edit</a></li>",
                    id = r.id
                ));
            }
        }
        _ => h.push_str("<li>No drafts yet.</li>"),
    }
    h.push_str("</ul>");
    postbox("dashboard_quick_press", "Quick Draft", h)
}

/// One Site Health check: (label, pass, detail).
struct HealthCheck {
    label: &'static str,
    pass: bool,
    detail: String,
}

/// Static verdict inputs (phase spec): db file < 200MB, `settings.last_backup`
/// within 7 days, pending comments < 100.
fn health_checks(c: &Connection, sc: &SiteCtx, pending: i64) -> Vec<HealthCheck> {
    let db_bytes = std::fs::metadata(&sc.db).map(|m| m.len()).unwrap_or(0);
    let db_mb = db_bytes / (1024 * 1024);
    let backup_raw = c
        .query_row(
            sql!("render__select_value_from_settings"),
            ["last_backup"],
            |r| r.get::<_, String>(0),
        )
        .unwrap_or_default();
    let backup_ok = backup_fresh(&backup_raw, 7);
    vec![
        HealthCheck {
            label: "Database size under 200MB",
            pass: db_bytes < 200 * 1024 * 1024,
            detail: format!("{db_mb} MB"),
        },
        HealthCheck {
            label: "Backup within the last 7 days",
            pass: backup_ok,
            detail: if backup_raw.trim().is_empty() {
                "no settings.last_backup recorded".into()
            } else {
                format!("last_backup={backup_raw}")
            },
        },
        HealthCheck {
            label: "Pending comments under 100",
            pass: pending < 100,
            detail: format!("{pending} pending"),
        },
    ]
}

/// True when `raw` parses as `YYYY-MM-DD[ HH:MM:SS]` or unix seconds and is
/// ≤ `days` old.
fn backup_fresh(raw: &str, days: i64) -> bool {
    let raw = raw.trim();
    if raw.is_empty() {
        return false;
    }
    let now_days = {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0) as i64;
        secs / 86_400
    };
    if let Ok(epoch) = raw.parse::<i64>() {
        return now_days - epoch / 86_400 <= days;
    }
    // ISO date: days-from-civil (Howard Hinnant's algorithm).
    let day = raw.get(..10).unwrap_or(raw);
    let mut it = day.split('-');
    let (Some(y), Some(m), Some(d)) = (
        it.next().and_then(|x| x.parse::<i64>().ok()),
        it.next().and_then(|x| x.parse::<i64>().ok()),
        it.next().and_then(|x| x.parse::<i64>().ok()),
    ) else {
        return false;
    };
    let y_adj = if m <= 2 { y - 1 } else { y };
    let era = if y_adj >= 0 { y_adj } else { y_adj - 399 } / 400;
    let yoe = y_adj - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let z = era * 146_097 + doe - 719_468;
    now_days - z <= days
}

/// Site Health Status body: "Good"/"Should be improved" + check bullets +
/// link to the full `/admin/health` screen.
fn health_box(c: &Connection, sc: &SiteCtx, pending: i64) -> String {
    let checks = health_checks(c, sc, pending);
    let good = checks.iter().all(|x| x.pass);
    let (verdict, color) = if good {
        ("Good", "#00a32a")
    } else {
        ("Should be improved", "#dba617")
    };
    let mut items = String::new();
    for chk in &checks {
        let icon = if chk.pass { "✔" } else { "⚠" };
        items.push_str(&format!(
            "<li>{icon} {} <span class=\"description\">({})</span></li>",
            chk.label,
            esc(&chk.detail)
        ));
    }
    postbox(
        "dashboard_site_health",
        "Site Health Status",
        format!(
            r#"<div class="site-health-progress"><span class="site-health-progress-label" style="color:{color};font-size:2em;font-weight:600">{verdict}</span></div>
<ul>{items}</ul>
<p><a href="/admin/health">Site Health screen</a></p>"#,
        ),
    )
}

/// GET /admin — WP `index.php`: screen options + N-column widget layout.
pub(crate) async fn dashboard(State(app): State<App>, headers: HeaderMap) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    let Ok(c) = crate::db::conn(&sc.db) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let opts = dash_opts(sc, s.user_id);
    let pending = crate::db::pending_comments_count(&sc.db);

    // WP widget order; hidden ones are skipped, visible ones deal
    // round-robin across the configured column count.
    let boxes = [
        ("dashboard_right_now", glance_box(&c)),
        ("dashboard_activity", activity_box(&c, sc, &s.csrf)),
        ("dashboard_quick_press", quick_draft_box(sc, &s)),
        ("dashboard_site_health", health_box(&c, sc, pending)),
    ];
    let cols = opts.columns.clamp(1, 4) as usize;
    let mut columns = vec![String::new(); cols];
    let mut i = 0usize;
    for (id, html) in boxes {
        if opts.hidden.iter().any(|h| h == id) {
            continue;
        }
        columns[i % cols].push_str(&html);
        i += 1;
    }
    let mut containers = String::new();
    for (n, col) in columns.iter().enumerate() {
        containers.push_str(&format!(
            "<div id=\"postbox-container-{}\" class=\"postbox-container\"><div class=\"meta-box-sortables\">{col}</div></div>",
            n + 1
        ));
    }

    // Widget edits are DB-only until the next full build — surface the
    // pending state WP's immediate-publish model can't on a static site.
    let widgets_dirty = crate::db::setting_value(&sc.db, "widgets_dirty", "0") == "1";
    let pending_notice = if widgets_dirty {
        r#"<div class="notice notice-warning"><p><strong>Widget changes are pending publish.</strong> Run Generate to update the site.</p></div>"#
    } else {
        ""
    };

    let body = format!(
        r#"{}<div class="wrap">{screen_options}<h1>Dashboard</h1>
{pending_notice}
<div id="dashboard-widgets-wrap"><div id="dashboard-widgets" class="metabox-holder columns-{cols}">
{containers}
</div></div>
<form method="post" action="/api/generate" style="margin-top:12px"><input type="hidden" name="csrf" value="{csrf}"><button class="button button-primary">Rebuild site files</button> <span class="description" style="margin-left:8px">Emits sitemap / feeds / robots / search / redirects + clears the page cache (it re-warms itself from traffic).</span></form>
</div>"#,
        chrome("dashboard", &s),
        screen_options = screen_options(&opts, &s.csrf),
        pending_notice = pending_notice,
        cols = cols,
        containers = containers,
        csrf = s.csrf,
    );
    html_admin(&s, &format!("Dashboard ‹ {}", s.site_name), &body, true, "index").into_response()
}

/// POST /admin/draft — Quick Draft save: creates a `status='draft'` post
/// owned by the session user, lands on the editor.
pub(crate) async fn save_draft(
    State(app): State<App>,
    headers: HeaderMap,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if !ct_eq(s.csrf.as_bytes(), f.get("csrf").map(|x| x.as_str()).unwrap_or("").as_bytes()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let title = f.get("title").cloned().unwrap_or_default();
    let content = f.get("content").cloned().unwrap_or_default();
    match crud::misc::insert_draft(&sc.db, &title, &content, s.user_id) {
        Ok(id) => Redirect::to(&format!("/admin/edit/{id}")).into_response(),
        Err(_) => Redirect::to("/admin").into_response(),
    }
}

/// POST /admin — Screen Options apply: persists widget visibility +
/// column layout to `settings.dashboard_{user_id}`.
pub(crate) async fn screen_options_save(
    State(app): State<App>,
    headers: HeaderMap,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if !ct_eq(s.csrf.as_bytes(), f.get("csrf").map(|x| x.as_str()).unwrap_or("").as_bytes()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let columns = f
        .get("columns")
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(2)
        .clamp(1, 4);
    // Unchecked checkboxes post nothing → absent `-hide` key means hidden.
    let hidden: Vec<String> = WIDGETS
        .iter()
        .map(|(id, _)| *id)
        .filter(|id| !f.contains_key(&format!("{id}-hide")))
        .map(|id| id.to_string())
        .collect();
    let value = serde_json::json!({ "columns": columns, "hidden": hidden }).to_string();
    let _ = crud::comments::setting_set(&sc.db, &format!("dashboard_{}", s.user_id), &value);
    Redirect::to("/admin").into_response()
}

/// GET /admin/health — Site Health screen. HTML page for browsers
/// (Accept: text/html), JSON otherwise.
pub(crate) async fn health_page(State(app): State<App>, headers: HeaderMap) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    let c = match crate::db::conn(&sc.db) {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({"status": "error"})),
            )
                .into_response()
        }
    };
    let pending = crate::db::pending_comments_count(&sc.db);
    let checks = health_checks(&c, sc, pending);
    let good = checks.iter().all(|x| x.pass);

    let wants_html = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|a| a.contains("text/html"))
        .unwrap_or(false);
    if !wants_html {
        return axum::Json(serde_json::json!({
            "status": if good { "good" } else { "should-be-improved" },
            "checks": checks
                .iter()
                .map(|x| serde_json::json!({"label": x.label, "pass": x.pass, "detail": x.detail}))
                .collect::<Vec<_>>(),
        }))
        .into_response();
    }

    let (verdict, color) = if good {
        ("Good", "#00a32a")
    } else {
        ("Should be improved", "#dba617")
    };
    let mut items = String::new();
    for chk in &checks {
        let (icon, cls) = if chk.pass { ("✔", "pass") } else { ("⚠", "warning") };
        items.push_str(&format!(
            "<tr><td>{icon} <span class=\"{cls}\">{}</span></td><td>{}</td></tr>",
            chk.label,
            esc(&chk.detail)
        ));
    }
    let body = format!(
        r#"{}<div class="wrap"><h1>Site Health</h1>
<div class="postbox"><h2 class="hndle">Site Health Status</h2><div class="inside">
<p><span style="color:{color};font-size:2em;font-weight:600">{verdict}</span></p>
<table class="widefat striped"><tbody>{items}</tbody></table>
</div></div>
</div>"#,
        chrome("dashboard", &s)
    );
    html_page(&s, &format!("Site Health ‹ {}", s.site_name), &body, true).into_response()
}
