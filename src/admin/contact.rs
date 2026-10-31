//! Contact → CF7 "Contact Forms" list + Flamingo "Inbound Messages" inbox.
//! Forms come from the `forms` table (imported CF7 forms); submissions are
//! moderated through `intake::moderate` so `approve` keeps feeding the
//! comments/reviews tables.
use super::list;
use super::*;
use crate::db::submissions as subdb;
use crate::db::forms as fdb;
use axum::http::Uri;
use serde_json::Value;

const INBOX: &str = "/admin/contact/inbox";

/// Hard cap on CSV export rows (Flamingo-scale; overflow gets a note row).
const EXPORT_CAP: i64 = 10_000;

fn if_empty<'a>(v: &'a str, d: String) -> String {
    if v.is_empty() { d } else { v.to_string() }
}

fn humanize(slug: &str) -> String {
    let s = slug.replace('-', " ");
    let mut c = s.chars();
    match c.next() {
        Some(ch) => format!("{}{}", ch.to_uppercase(), c.as_str()),
        None => String::new(),
    }
}

/// Inbox "From" cell: `name` from data_json, falling back to the name column.
fn from_of(r: &subdb::Submission) -> String { // gate-allow: blocking by contract — only called inside spawn_blocking
    let map: serde_json::Map<String, Value> = serde_json::from_str(&r.data_json).unwrap_or_default();
    let name = map.get("name").and_then(|v| v.as_str()).unwrap_or(r.name.as_str());
    let email = map.get("email").and_then(|v| v.as_str()).unwrap_or("");
    if email.is_empty() {
        esc(name)
    } else {
        format!("{} &lt;<a href=\"mailto:{}\">{}</a>&gt;", esc(name), esc(email), esc(email))
    }
}

/// Subject heuristic: prefer `subject`/`name` keys, else first non-`_` field
/// (first line, ≤60 chars).
fn subject_of(data_json: &str) -> String {
    let map: serde_json::Map<String, Value> = serde_json::from_str(data_json).unwrap_or_default();
    for key in ["subject", "name"] {
        if let Some(v) = map.get(key).and_then(|v| v.as_str()) {
            if !v.trim().is_empty() {
                return trunc(v.trim(), 60);
            }
        }
    }
    for (k, v) in &map {
        if k.starts_with('_') {
            continue;
        }
        let s = v.as_str().unwrap_or("").trim();
        if !s.is_empty() {
            return trunc(s, 60);
        }
    }
    String::new()
}

fn trunc(s: &str, n: usize) -> String {
    let first = s.lines().next().unwrap_or(s);
    if first.chars().count() > n {
        format!("{}…", first.chars().take(n).collect::<String>())
    } else {
        first.to_string()
    }
}

/// Loose `YYYY-MM-DD` guard for the from/to filter inputs — anything else
/// is dropped so the SQL `created_at` lexicographic bounds stay sane.
fn date_ok(s: &str) -> String {
    let ok = s.len() == 10
        && s.bytes().enumerate().all(|(i, b)| {
            if i == 4 || i == 7 { b == b'-' } else { b.is_ascii_digit() }
        });
    if ok { s.to_string() } else { String::new() }
}


/// Sorted `k=v&…` query string of the current request minus `msg`.
fn qs_pairs(q: &HashMap<String, String>) -> String {
    let mut pairs: Vec<(&String, &String)> = q.iter().filter(|(k, _)| k.as_str() != "msg").collect();
    pairs.sort();
    pairs.iter().map(|(k, v)| format!("{}={}", urlenc(k), urlenc(v))).collect::<Vec<_>>().join("&")
}

/// Current query string minus `msg` — the `back` target for row actions.
fn qs_preserve(q: &HashMap<String, String>) -> String {
    let s = qs_pairs(q);
    if s.is_empty() { INBOX.to_string() } else { format!("{INBOX}?{s}") }
}

/// Extra params kept by view links / pager links.
fn extra(q: &HashMap<String, String>, keep_view: bool) -> HashMap<String, String> {
    let mut m = HashMap::new();
    for k in ["form", "s", "sort", "dir", "from", "to"] {
        if let Some(v) = q.get(k) {
            if !v.is_empty() {
                m.insert(k.to_string(), v.clone());
            }
        }
    }
    if keep_view {
        if let Some(v) = q.get("view") {
            if !v.is_empty() && v != "inbox" {
                m.insert("view".into(), v.clone());
            }
        }
    }
    m
}

fn sort_href(q: &HashMap<String, String>, col: &str, cur_sort: &str, cur_dir: &str) -> String {
    let dir = if cur_sort == col {
        if cur_dir == "asc" { "desc" } else { "asc" }
    } else if col == "date" {
        "desc"
    } else {
        "asc"
    };
    let mut pairs = vec![
        format!("sort={col}"),
        format!("dir={dir}"),
    ];
    for k in ["view", "form", "s", "from", "to"] {
        if let Some(v) = q.get(k) {
            if !v.is_empty() && !(k == "view" && v == "inbox") {
                pairs.push(format!("{k}={}", urlenc(v)));
            }
        }
    }
    format!("{INBOX}?{}", pairs.join("&"))
}

/// (verb, label, css class, confirm) row/bulk actions for a view.
fn view_actions(view: &str, kind: &str, status: &str) -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
    let del = "Permanently delete this submission?";
    match view {
        "spam" => vec![
            ("restore", "Not Spam", "unspam", ""),
            ("purge", "Delete Permanently", "delete", del),
        ],
        "trash" => vec![
            ("restore", "Restore", "restore", ""),
            ("purge", "Delete Permanently", "delete", del),
        ],
        _ => {
            // 'Approve' only helps once — comment/review rows already approved
            // via moderate() would duplicate on re-approve.
            let mut v: Vec<(&'static str, &'static str, &'static str, &'static str)> = Vec::new();
            if matches!(kind, "comment" | "review") && !matches!(status, "approved") {
                v.push(("approve", "Approve", "approve", ""));
            }
            v.push(("spam", "Mark as Spam", "spam", ""));
            v.push(("trash", "Trash", "trash", ""));
            v
        }
    }
}

/// `.row-actions` markup: inbox rows get a View link first; POST forms carry
/// the per-view verbs (WP uses GET+nonce; we use POST+CSRF like
/// `list::row_actions`, plus a named `action` submit button for the verb).
fn inbox_row_actions(s: &Sess, r: &subdb::Submission, view: &str, back: &str) -> String { // gate-allow: blocking by contract — only called inside spawn_blocking
    let mut out = String::from("<div class=\"row-actions\">");
    let mut first = true;
    if view == "inbox" {
        out.push_str(&format!(
            "<span class=\"view\"><a href=\"{INBOX}/{}\">View</a></span>",
            r.id
        ));
        first = false;
    }
    for (verb, label, class, confirm) in view_actions(view, &r.kind, &r.status) {
        if !first {
            out.push_str(" | ");
        }
        first = false;
        let onclick = if confirm.is_empty() {
            String::new()
        } else {
            format!(" onclick=\"return confirm('{}')\"", esc(confirm))
        };
        out.push_str(&format!(
            "<span class=\"{class}\"><form method=\"post\" action=\"{INBOX}/action\" style=\"display:inline\">\
             <input type=\"hidden\" name=\"csrf\" value=\"{csrf}\">\
             <input type=\"hidden\" name=\"id\" value=\"{id}\">\
             <input type=\"hidden\" name=\"back\" value=\"{back}\">\
             <button type=\"submit\" name=\"action\" value=\"{verb}\" class=\"button-link\"{onclick}>{label}</button></form></span>",
            class = class,
            csrf = s.csrf,
            id = r.id,
            back = esc(back),
            verb = verb,
            onclick = onclick,
            label = esc(label)
        ));
    }
    out.push_str("</div>");
    out
}

fn notice(msg: &str) -> String {
    let text = match msg {
        "approve" => "Submission approved.",
        "spam" => "Marked as spam.",
        "trash" => "Moved to Trash.",
        "restore" => "Submission restored.",
        "purge" => "Submission permanently deleted.",
        _ => return String::new(),
    };
    format!("<div class=\"notice notice-success is-dismissible\"><p>{text}</p></div>")
}

/// Screen A: `/admin/contact` — CF7 Contact Forms parity.
/// Lists `forms` table rows; title links to `/edit` (CF7-style editor).
pub(crate) async fn forms_page(State(app): State<App>, headers: HeaderMap) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    // rusqlite is sync — offload the whole page build (counts + forms
    // queries, table HTML) to a blocking thread.
    let db = sc.db.clone();
    tokio::task::spawn_blocking(move || {
        let counts = subdb::form_counts(&db).unwrap_or_default();
        let forms = fdb::list(&db).unwrap_or_default();
        let mut rows = String::new();
        if forms.is_empty() {
            rows.push_str(&list::empty_row(6, "No contact forms found."));
        }
        for r in &forms {
            let slug = r.slug.as_str();
            let n = counts.get(slug).copied().unwrap_or(0);
            rows.push_str(&format!(
                "<tr id=\"form-{slug}\">\
                 <th scope=\"row\" class=\"check-column\"><input type=\"checkbox\" name=\"p-{slug}\" disabled></th>\
                 <td class=\"title column-title column-primary\"><strong><a class=\"row-title\" href=\"/admin/contact/form/{slug}/edit\">{title}</a></strong>\
                 <div class=\"row-actions\"><span class=\"edit\"><a href=\"/admin/contact/form/{slug}/edit\">Edit</a></span> | <span class=\"view\"><a href=\"/admin/contact/form/{slug}\">View</a></span> | <span class=\"submissions\"><a href=\"{INBOX}?form={slug}\">Submissions</a></span> | <span class=\"delete\"><button type=\"submit\" class=\"button-link submitdelete\" form=\"form-del\" name=\"slug\" value=\"{slug}\" onclick=\"return confirm('Permanently delete this contact form?')\">Delete</button></span></div></td>\
                 <td class=\"shortcode\"><code>[contact-form-7 id=\"{slug}\" title=\"{title_attr}\"]</code></td>\
                 <td class=\"submissions\"><a href=\"{INBOX}?form={slug}\">{n}</a></td>\
                 <td class=\"date\">—</td>\
                 <td class=\"lang\">{lang}</td></tr>",
                slug = slug,
                title = esc(&r.title),
                title_attr = esc(&r.title),
                n = n,
                lang = r.lang,
            ));
        }
        let body = format!(
            "{}<div class=\"wrap\"><h1 class=\"wp-heading-inline\">Contact Forms</h1>\
             <a href=\"/admin/contact/new\" class=\"page-title-action\">Add New</a>\
             <hr class=\"wp-header-end\">\
             <table class=\"wp-list-table widefat fixed striped table-view-list\">\
             <thead><tr>\
             <td class=\"manage-column column-cb check-column\"><input type=\"checkbox\" disabled></td>\
             <th scope=\"col\" class=\"manage-column column-title column-primary\">Title</th>\
             <th scope=\"col\" class=\"manage-column column-shortcode\">Shortcode</th>\
             <th scope=\"col\" class=\"manage-column column-submissions num\">Submissions</th>\
             <th scope=\"col\" class=\"manage-column column-date sortable desc\"><a href=\"#\"><span>Date</span></a></th>\
             <th scope=\"col\" class=\"manage-column column-lang\">Lang</th>\
             </tr></thead><tbody>{}</tbody></table>\
             <form method=\"post\" action=\"/admin/contact/form/delete\" id=\"form-del\"><input type=\"hidden\" name=\"csrf\" value=\"{}\"></form></div>",
            chrome("contact", &s),
            rows,
            s.csrf
        );
        html_admin(&s, "Contact Forms", &body, true, "edit-php").into_response()
    })
    .await
    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
/// `POST /admin/contact/form/delete` — drop a `forms` row by slug. Submissions
/// keep their `form_slug` (orphaned, not deleted — same as WP/CF7).
pub(crate) async fn form_delete(
    State(app): State<App>,
    headers: HeaderMap,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    if !ct_eq(s.csrf.as_bytes(), f.get("csrf").map(|x| x.as_str()).unwrap_or("").as_bytes()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let slug = f.get("slug").cloned().unwrap_or_default();
    // rusqlite is sync — offload the forms lookup + delete to a
    // blocking thread.
    let db = sc.db.clone();
    let _ = tokio::task::spawn_blocking(move || {
        if let Ok(Some(row)) = fdb::get_by_slug(&db, &slug) {
            let _ = fdb::delete(&db, row.id);
        }
    })
    .await;
    Redirect::to("/admin/contact").into_response()
}


/// Screen B: `/admin/contact/inbox` — Flamingo Inbound Messages parity.
pub(crate) async fn inbox_page(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    let view = q.get("view").map(|s| s.as_str()).unwrap_or("inbox");
    let view = if matches!(view, "spam" | "trash") { view } else { "inbox" };
    let form = q.get("form").cloned().unwrap_or_default();
    let kind = q.get("kind").cloned().unwrap_or_default();
    let search = q.get("s").cloned().unwrap_or_default();
    let sort = q.get("sort").cloned().unwrap_or_default();
    let sort = if matches!(sort.as_str(), "subject" | "from" | "date") { sort } else { "date".to_string() };
    let dir = q.get("dir").cloned().unwrap_or_default();
    let dir = match dir.as_str() {
        "asc" => "asc".to_string(),
        "desc" => "desc".to_string(),
        _ => if sort == "date" { "desc" } else { "asc" }.to_string(),
    };

    let from = date_ok(q.get("from").map(|s| s.as_str()).unwrap_or(""));
    let to = date_ok(q.get("to").map(|s| s.as_str()).unwrap_or(""));

    // `?export=1` short-circuit — covers `/admin/reviews?export=1` too
    // (reviews_page injects kind=review before delegating here).
    if q.get("export").is_some() {
        return inbox_export(State(app), headers, Query(q)).await;
    }

    // rusqlite is sync — offload the counts + both list queries + the
    // forms dropdown to a blocking thread. Count first so Pager knows
    // `total` before the bounded query runs.
    let db = sc.db.clone();
    let (c_view, c_form, c_kind, c_search, c_from, c_to, c_sort, c_dir, c_q, slug2) = (
        view.to_string(),
        form.clone(),
        kind.clone(),
        search.clone(),
        from.clone(),
        to.clone(),
        sort.clone(),
        dir.clone(),
        q.clone(),
        form.clone(),
    );
    let data = tokio::task::spawn_blocking(move || {
        let counts = subdb::counts(&db).unwrap_or_default();
        let pre = subdb::Filter {
            status: c_view.clone(), form_slug: c_form.clone(), kind: c_kind.clone(), s: c_search.clone(),
            limit: 0, offset: 0, from: c_from.clone(), to: c_to.clone(), sort: c_sort.clone(), dir: c_dir.clone(),
        };
        let (_, total) = subdb::list(&db, &pre).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
        let pager = list::Pager::new(&c_q, total);
        let flt = subdb::Filter {
            status: c_view, form_slug: c_form, kind: c_kind, s: c_search,
            limit: pager.limit(), offset: pager.offset(), from: c_from, to: c_to, sort: c_sort, dir: c_dir,
        };
        let (rows, _) = subdb::list(&db, &flt).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
        let mut opts = String::from("<option value=\"\">View all forms</option>");
        if let Ok(forms) = crate::db::forms::list(&db) {
            for f in &forms {
                let sel = if f.slug == slug2 { " selected" } else { "" };
                opts.push_str(&format!(
                    "<option value=\"{}\"{}>{}</option>",
                    esc(&f.slug),
                    sel,
                    esc(&if_empty(&f.title, humanize(&f.slug)))
                ));
            }
        }
        Ok::<(HashMap<String, i64>, list::Pager, Vec<subdb::Submission>, String), Response>((counts, pager, rows, opts))
    })
    .await;
    let Ok(Ok((counts, pager, rows, opts))) = data else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let inbox_n = counts.get("new").copied().unwrap_or(0) + counts.get("pending").copied().unwrap_or(0);
    let spam_n = counts.get("spam").copied().unwrap_or(0);
    let trash_n = counts.get("trash").copied().unwrap_or(0) + counts.get("delete").copied().unwrap_or(0);

    let extra_views = extra(&q, false);
    let extra_pager = extra(&q, true);
    let views = list::views_bar(
        "/admin/contact/inbox?",
        "view",
        view,
        &[
            list::View { key: "inbox", label: "Inbox", count: inbox_n },
            list::View { key: "spam", label: "Spam", count: spam_n },
            list::View { key: "trash", label: "Trash", count: trash_n },
        ],
        &extra_views,
    );

    // Filter (GET): form dropdown + search box + Filter button.
    let view_hidden = if view == "inbox" {
        String::new()
    } else {
        format!("<input type=\"hidden\" name=\"view\" value=\"{view}\">")
    };
    let export_href = format!("{INBOX}/export?{}", qs_pairs(&q));
    let filter_form = format!(
        "<form method=\"get\" action=\"{INBOX}\" id=\"inbox-filter\">{view_hidden}\
         <div class=\"tablenav top\">\
         <p class=\"search-box\"><label class=\"screen-reader-text\" for=\"post-search-input\">Search Inbound Messages:</label>\
         <input type=\"search\" id=\"post-search-input\" name=\"s\" value=\"{}\">\
         <input type=\"submit\" class=\"button\" value=\"Search Inbound Messages\"></p>\
         <div class=\"alignleft actions\"><label class=\"screen-reader-text\" for=\"filter-by-form\">Filter by form</label>\
         <select name=\"form\" id=\"filter-by-form\">{}</select>\
         <label class=\"screen-reader-text\" for=\"filter-from\">From</label>\
         <input type=\"date\" name=\"from\" id=\"filter-from\" value=\"{}\">\
         <label class=\"screen-reader-text\" for=\"filter-to\">To</label>\
         <input type=\"date\" name=\"to\" id=\"filter-to\" value=\"{}\">\
         <input type=\"submit\" class=\"button\" value=\"Filter\">\
         <a class=\"button\" href=\"{}\">Export</a></div>\
         <br class=\"clear\"></div></form>",
        esc(&search),
        opts,
        esc(&from),
        esc(&to),
        esc(&export_href)
    );

    // Bulk actions available on the current view.
    let bulk: &[(&str, &str)] = match view {
        "spam" => &[("restore", "Not Spam"), ("purge", "Delete Permanently")],
        "trash" => &[("restore", "Restore"), ("purge", "Delete Permanently")],
        _ => &[("trash", "Move to Trash"), ("spam", "Mark as Spam")],
    };

    let s_sub = sort_href(&q, "subject", &sort, &dir);
    let s_from = sort_href(&q, "from", &sort, &dir);
    let s_date = sort_href(&q, "date", &sort, &dir);
    let head = list::thead(&[
        ("title column-primary", "Subject", s_sub.as_str()),
        ("from", "From", s_from.as_str()),
        ("form", "Form", ""),
        ("date", "Date", s_date.as_str()),
    ]);

    let back = qs_preserve(&q);
    let mut body_rows = String::new();
    for r in &rows {
        let subject = subject_of(&r.data_json);
        let subject = if subject.is_empty() { "(no subject)".to_string() } else { esc(&subject) };
        let form_cell = if r.form_slug.is_empty() { "—".to_string() } else { esc(&r.form_slug) };
        body_rows.push_str(&format!(
            "<tr id=\"submission-{id}\">\
             <th scope=\"row\" class=\"check-column\"><input form=\"inbox-bulk\" type=\"checkbox\" name=\"p-{id}\"></th>\
             <td class=\"title column-title column-primary has-row-actions\"><strong>\
             <a class=\"row-title\" href=\"{INBOX}/{id}\">{subject}</a></strong>{actions}</td>\
             <td class=\"from\">{from}</td>\
             <td class=\"form\">{form_cell}</td>\
             <td class=\"date\">{date}</td></tr>",
            id = r.id,
            subject = subject,
            actions = inbox_row_actions(&s, r, view, &back),
            from = from_of(r),
            form_cell = form_cell,
            date = esc(&r.created_at)
        ));
    }
    if rows.is_empty() {
        body_rows.push_str(&list::empty_row(5, "No submissions found."));
    }

    let (title, menu) = if kind == "review" { ("Site Reviews", "reviews") } else { ("Inbound Messages", "contact") };
    let body = format!(
        "{}<div class=\"wrap\"><h1 class=\"wp-heading-inline\">{}</h1>\
         <hr class=\"wp-header-end\">{}{}{}\
         <form method=\"post\" action=\"{INBOX}/bulk\" id=\"inbox-bulk\">\
         <input type=\"hidden\" name=\"csrf\" value=\"{}\">\
         <input type=\"hidden\" name=\"view\" value=\"{}\">\
         <input type=\"hidden\" name=\"kind\" value=\"{}\">\
         <input type=\"hidden\" name=\"back\" value=\"{}\">\
         {}\
         <table class=\"wp-list-table widefat fixed striped table-view-list\">{}<tbody>{}</tbody></table>\
         {}</form></div>",
        chrome(menu, &s),
        title,
        notice(q.get("msg").map(|s| s.as_str()).unwrap_or("")),
        views,
        filter_form,
        s.csrf,
        view,
        esc(&kind),
        esc(&back),
        list::tablenav(true, bulk, Some(&pager), INBOX, &extra_pager),
        head,
        body_rows,
        list::tablenav(false, bulk, None, INBOX, &extra_pager)
    );
    html_admin(&s, title, &body, true, "edit-php").into_response()
}

/// Screen C: `/admin/contact/inbox/{id}` — submission detail.
pub(crate) async fn detail_page(State(app): State<App>, headers: HeaderMap, Path(id): Path<i64>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    // rusqlite is sync — offload the fetch + page build to a blocking thread.
    let db = sc.db.clone();
    tokio::task::spawn_blocking(move || {
        let Some(r) = subdb::get(&db, id).unwrap_or(None) else {
            return StatusCode::NOT_FOUND.into_response();
        };
        let view = match r.status.as_str() {
            "spam" => "spam",
            "trash" | "delete" => "trash",
            _ => "inbox",
        };
        let map: serde_json::Map<String, Value> = serde_json::from_str(&r.data_json).unwrap_or_default();
        let subject = subject_of(&r.data_json);
        let subject = if subject.is_empty() { "(no subject)".to_string() } else { subject };

        // Meta header: From / Form / Date / IP / Page / Tour / Kind / Status.
        let meta_row = |k: &str, v: String| format!("<tr><th scope=\"row\">{k}</th><td>{v}</td></tr>");
        let mut meta = String::new();
        meta.push_str(&meta_row("From", from_of(&r)));
        meta.push_str(&meta_row("Form", if r.form_slug.is_empty() { "—".into() } else { esc(&r.form_slug) }));
        meta.push_str(&meta_row("Date", esc(&r.created_at)));
        meta.push_str(&meta_row("IP", if r.ip.is_empty() { "—".into() } else { esc(&r.ip) }));
        meta.push_str(&meta_row("Page", if r.page.is_empty() { "—".into() } else { esc(&r.page) }));
        meta.push_str(&meta_row("Tour", if r.tour.is_empty() { "—".into() } else { esc(&r.tour) }));
        meta.push_str(&meta_row("Kind", esc(&r.kind)));
        meta.push_str(&meta_row("Status", esc(&r.status)));

        let mut kv = String::new();
        for (k, v) in &map {
            let val = v.as_str().map(|x| x.to_string()).unwrap_or_else(|| v.to_string());
            kv.push_str(&format!(
                "<tr><th scope=\"row\">{}</th><td>{}</td></tr>",
                esc(k),
                esc(&val)
            ));
        }
        if kv.is_empty() {
            kv.push_str("<tr><td colspan=\"2\">No fields recorded.</td></tr>");
        }

        // Action buttons mirror the row actions of this row's current view.
        let mut buttons = String::new();
        for (verb, label, _class, confirm) in view_actions(view, &r.kind, &r.status) {
            let onclick = if confirm.is_empty() {
                String::new()
            } else {
                format!(" onclick=\"return confirm('{}')\"", esc(confirm))
            };
            buttons.push_str(&format!(
                "<form method=\"post\" action=\"{INBOX}/action\" style=\"display:inline\">\
                 <input type=\"hidden\" name=\"csrf\" value=\"{}\">\
                 <input type=\"hidden\" name=\"id\" value=\"{}\">\
                 <input type=\"hidden\" name=\"back\" value=\"{INBOX}?view={view}\">\
                 <button type=\"submit\" name=\"action\" value=\"{}\" class=\"button\"{}>{}</button></form> ",
                s.csrf,
                r.id,
                verb,
                onclick,
                esc(label)
            ));
        }

        let body = format!(
            "{}<div class=\"wrap\"><h1 class=\"wp-heading-inline\">Inbound Message</h1>\
             <a href=\"{INBOX}?view={view}\" class=\"page-title-action\">← Back to list</a>\
             <hr class=\"wp-header-end\">\
             <h2>{}</h2>\
             <table class=\"form-table\"><tbody>{}</tbody></table>\
             <h2>Fields</h2>\
             <table class=\"widefat striped\"><tbody>{}</tbody></table>\
             <p class=\"submit\">{}</p></div>",
            chrome("contact", &s),
            esc(&subject),
            meta,
            kv,
            buttons
        );
        html_page(&s, "Inbound Message", &body, true).into_response()
    })
    .await
    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// Screen D: `/admin/contact/form/{slug}` — CF7 "Edit Contact Form" parity.
/// The form itself lives in `forms.markup` (imported CF7 markup baked into
/// the binary), so the screen is read-only: slug, endpoint, lang, counts,
/// and the most recent submissions.
pub(crate) async fn form_detail(State(app): State<App>, headers: HeaderMap, Path(slug): Path<String>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    // rusqlite is sync — offload the form lookup + counts + recent-
    // submissions query and the page build to a blocking thread.
    let db = sc.db.clone();
    tokio::task::spawn_blocking(move || {
        let Ok(Some(form_row)) = crate::db::forms::get_by_slug(&db, &slug) else {
            return StatusCode::NOT_FOUND.into_response();
        };
        let counts = subdb::form_counts(&db).unwrap_or_default();
        let n = counts.get(slug.as_str()).copied().unwrap_or(0);
        let lang = crate::lang::lang_label(&form_row.lang);
        let mut meta = String::new();
        let mr = |k: &str, v: String| format!("<tr><th scope=\"row\">{k}</th><td>{v}</td></tr>");
        meta.push_str(&mr("Title", esc(&if_empty(&form_row.title, humanize(&slug)))));
        meta.push_str(&mr("Slug", esc(&slug)));
        meta.push_str(&mr("Endpoint", format!("<code>POST /form/{}</code>", esc(&slug))));
        meta.push_str(&mr("Lang", lang.to_string()));
        meta.push_str(&mr("Submissions", format!(
            "<a href=\"{INBOX}?form={}\">{}</a>",
            esc(&slug),
            n
        )));

        // Recent submissions for this form (last 10, newest first).
        let mut rows = String::new();
        match subdb::list(&db, &subdb::Filter {
            status: "".into(),
            form_slug: slug.clone(),
            s: String::new(),
            kind: "form".into(),
            limit: 10,
            offset: 0,
            from: String::new(),
            to: String::new(),
            sort: "date".into(),
            dir: "desc".into(),
        }) {
            Ok((rs, _total)) => {
                for r in &rs {
                    let subject = subject_of(&r.data_json);
                    let subject = if subject.is_empty() { "(no subject)".to_string() } else { esc(&subject) };
                    rows.push_str(&format!(
                        "<tr><td><a href=\"{INBOX}/{}\">{}</a></td><td>{}</td><td>{}</td><td>{}</td></tr>",
                        r.id,
                        subject,
                        from_of(r),
                        esc(&r.status),
                        esc(&r.created_at)
                    ));
                }
            }
            Err(_) => {}
        }
        if rows.is_empty() {
            rows.push_str("<tr><td colspan=\"4\">No submissions yet.</td></tr>");
        }

        let body = format!(
            "{}<div class=\"wrap\"><h1 class=\"wp-heading-inline\">Contact Form</h1>\
             <a href=\"/admin/contact\" class=\"page-title-action\">← Back to list</a>\
             <hr class=\"wp-header-end\">\
             <h2>{}</h2>\
             <table class=\"form-table\"><tbody>{}</tbody></table>\
             <h2>Recent Submissions</h2>\
             <table class=\"widefat striped\"><thead><tr><th>Subject</th><th>From</th><th>Status</th><th>Date</th></tr></thead><tbody>{}</tbody></table>\
             </div>",
            chrome("contact", &s),
            esc(&if_empty(&form_row.title, humanize(&slug))),
            meta,
            rows
        );
        html_admin(&s, "Contact Form", &body, false, "edit-php").into_response()
    })
    .await
    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// Screen E: `/admin/reviews` — Site Reviews inbox. Same table as Inbox,
/// restricted to `kind='review'` (rows land there via `intake::submit_review`).
pub(crate) async fn reviews_page(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let mut q2 = q;
    q2.insert("kind".into(), "review".into());
    inbox_page(State(app), headers, Query(q2)).await
}


/// GET `/admin/contact/inbox/export` — Flamingo "Export" action.
/// Streams the filtered set (same view/form/kind/search/date filters as the
/// inbox GET) as a CSV attachment, capped at 10,000 rows. `kind=review`
/// (carried from the Site Reviews screen) yields `reviews-*.csv`.
pub(crate) async fn inbox_export(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Ok((_s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    let view = q.get("view").map(|s| s.as_str()).unwrap_or("inbox");
    let status: String = if matches!(view, "spam" | "trash") { view } else { "inbox" }.to_string();
    let is_review = q.get("kind").map(|k| k.as_str()) == Some("review");
    // rusqlite is sync — offload the CSV export + date lookup to a
    // blocking thread.
    let db = sc.db.clone();
    let data = tokio::task::spawn_blocking(move || {
        let f = subdb::Filter {
            status,
            form_slug: q.get("form").cloned().unwrap_or_default(),
            kind: q.get("kind").cloned().unwrap_or_default(),
            s: q.get("s").cloned().unwrap_or_default(),
            from: date_ok(q.get("from").map(|s| s.as_str()).unwrap_or("")),
            to: date_ok(q.get("to").map(|s| s.as_str()).unwrap_or("")),
            limit: 0,
            offset: 0,
            sort: q.get("sort").cloned().unwrap_or_default(),
            dir: q.get("dir").cloned().unwrap_or_default(),
        };
        let csv = subdb::export_csv(&db, &f, EXPORT_CAP)?;
        let day: String = crate::db::conn(&db).ok()
            .and_then(|c| c.query_row(sql!("admin__select_today_ymd"), [], |r| r.get(0)).ok())
            .unwrap_or_default();
        Ok::<(String, String), anyhow::Error>((csv, day))
    })
    .await;
    let Ok(Ok((csv, day))) = data else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let base = if is_review { "reviews" } else { "inbound-messages" };
    (
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{base}-{day}.csv\"")),
        ],
        csv,
    )
        .into_response()
}
/// Valid single-row verbs → `intake::moderate` words.
fn verb_of(action: &str) -> Option<&'static str> {
    match action {
        "spam" => Some("spam"),
        "trash" | "delete" => Some("trash"),
        "restore" => Some("restore"),
        "purge" => Some("purge"),
        "approve" => Some("approve"),
        _ => None,
    }
}

/// POST `/admin/contact/inbox/action` — one row, one action.
pub(crate) async fn inbox_action(
    State(app): State<App>,
    headers: HeaderMap,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = check_csrf(&s, f.get("csrf").map(|x| x.as_str()).unwrap_or("")) {
        return r;
    }
    let back = list::back_or(&f, INBOX).to_string();
    let Some(verb) = f.get("action").and_then(|a| verb_of(a)) else {
        return Redirect::to(&back).into_response();
    };
    let id: i64 = f.get("id").and_then(|x| x.parse().ok()).unwrap_or(0);
    if id > 0 {
        // Approving a comment/review submission republishes its post page.
        // rusqlite is sync — moderate off the async thread.
        let db = sc.db.clone();
        let pid = tokio::task::spawn_blocking(move || intake::moderate(&db, id, verb))
            .await
            .unwrap_or_else(|_| Err(anyhow::anyhow!("join")));
        if let Ok(pid) = pid {
            admin::regen_posts(sc, &[pid]);
        }
    }
    let sep = if back.contains('?') { '&' } else { '?' };
    Redirect::to(&format!("{back}{sep}msg={verb}")).into_response()
}

/// POST `/admin/contact/inbox/bulk` — `p-{id}` checkboxes + contextual action.
pub(crate) async fn inbox_bulk(
    State(app): State<App>,
    headers: HeaderMap,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = check_csrf(&s, f.get("csrf").map(|x| x.as_str()).unwrap_or("")) {
        return r;
    }
    let back = list::back_or(&f, INBOX).to_string();
    let view = f.get("view").map(|x| x.as_str()).unwrap_or("inbox");
    let action = match f.get("action").map(|x| x.as_str()) {
        Some(a) if a != "-1" => a,
        _ => f.get("action2").map(|x| x.as_str()).unwrap_or("-1"),
    };
    // Contextual bulk map per view (Flamingo parity).
    let verb = match (view, action) {
        (_, "trash") => Some("trash"),
        (_, "spam") => Some("spam"),
        ("spam" | "trash", "restore") => Some("restore"),
        ("spam" | "trash", "purge") => Some("purge"),
        _ => None,
    };
    let Some(verb) = verb else {
        return Redirect::to(&back).into_response();
    };
    // rusqlite is sync — one blocking pass moderates every checked row.
    let db = sc.db.clone();
    let ids: Vec<i64> = f.keys()
        .filter_map(|k| k.strip_prefix("p-").and_then(|x| x.parse::<i64>().ok()))
        .collect();
    let posts: Vec<i64> = tokio::task::spawn_blocking(move || {
        ids.iter()
            .filter_map(|&id| intake::moderate(&db, id, verb).ok())
            .collect()
    })
    .await
    .unwrap_or_default();
    admin::regen_posts(sc, &posts);
    let sep = if back.contains('?') { '&' } else { '?' };
    Redirect::to(&format!("{back}{sep}msg={verb}")).into_response()
}

/// GET `/admin/submissions` → 301 `/admin/contact/inbox` (query preserved).
pub(crate) async fn submissions_redirect(uri: Uri) -> Response {
    let mut target = INBOX.to_string();
    if let Some(q) = uri.query() {
        if !q.is_empty() {
            target.push('?');
            target.push_str(q);
        }
    }
    Redirect::permanent(&target).into_response()
}

/// POST `/admin/submissions/mod` — legacy Moderation form → inbox action logic.
pub(crate) async fn submissions_mod_redirect(
    State(app): State<App>,
    headers: HeaderMap,
    Form(m): Form<intake::Moderation>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = check_csrf(&s, &m.csrf) {
        return r;
    }
    let msg = verb_of(&m.action).unwrap_or("");
    if !msg.is_empty() {
        // rusqlite is sync — moderate off the async thread.
        let db = sc.db.clone();
        let id = m.id;
        let pid = tokio::task::spawn_blocking(move || intake::moderate(&db, id, msg))
            .await
            .unwrap_or_else(|_| Err(anyhow::anyhow!("join")));
        if let Ok(pid) = pid {
            admin::regen_posts(sc, &[pid]);
        }
    }
    let sep = if msg.is_empty() { "" } else { "?msg=" };
    Redirect::to(&format!("{INBOX}{sep}{msg}")).into_response()
}
