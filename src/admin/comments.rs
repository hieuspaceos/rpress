//! WP `edit-comments.php` + `comment.php` parity.
//!
//! Intentional parity note (phase-2 spec): the comment-type `<select>` is a
//! visual no-op — there is no pingback/trackback `type` column in the schema,
//! so submitting it filters nothing. "Mine" works through `c.email` = the
//! session user's email (comments carry no user FK). "All" = approved +
//! pending.
//!
//! Row-action POSTs: `<form>` inside `<tbody>` is dropped by parsers, so all
//! mutation buttons carry `form="cmt-act"` and point at a single out-of-table
//! form via `formaction="/admin/comments/action?action=VERB&id=N"`; the verb
//! and id ride the query string, csrf/back ride the shared form. Bulk selects
//! keep WP's `action`/`action2` names (top/bottom).
use super::*;

type Cmt = crud::comments::CommentRow;

/// Percent-encode a query-string value (list::urlenc is private).
fn urlenc(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// `esc` plus `&#10;`/`&#13;` for newlines — attribute values carrying
/// multiline comment bodies (HTML normalizes raw CR/LF to spaces in attrs).
fn esc_attr(s: &str) -> String {
    esc(s).replace('\n', "&#10;").replace('\r', "&#13;")
}

/// Current list URL (`/admin/comments?status=…`) for `back` preservation.
fn back_url(q: &HashMap<String, String>) -> String {
    let mut u = String::from("/admin/comments");
    let mut first = true;
    for k in ["status", "post_id", "s", "paged"] {
        if let Some(v) = q.get(k).filter(|v| !v.is_empty()) {
            u.push_str(if first { "?" } else { "&" });
            first = false;
            u.push_str(&format!("{k}={}", urlenc(v)));
        }
    }
    u
}

/// Redirect back to the list with a `?m=` notice + affected count.
fn redirect_back(back: &str, m: &str, n: i64) -> Response {
    let sep = if back.contains('?') { "&" } else { "?" };
    Redirect::to(&format!("{back}{sep}m={m}&n={n}")).into_response()
}

/// `?m=&n=` → WP-style `<div class="notice">` ("1 comment approved").
fn notice_html(q: &HashMap<String, String>) -> String {
    let Some(m) = q.get("m").map(|s| s.as_str()) else {
        return String::new();
    };
    let n: i64 = q.get("n").and_then(|v| v.parse().ok()).unwrap_or(1).max(0);
    let word = if n == 1 { "comment" } else { "comments" };
    let (class, text) = match m {
        "approved" => ("notice-success", format!("{n} {word} approved")),
        "unapproved" => ("notice-success", format!("{n} {word} unapproved")),
        "spammed" => ("notice-success", format!("{n} {word} marked as spam")),
        "unspammed" => ("notice-success", format!("{n} {word} removed from spam")),
        "trashed" => ("notice-success", format!("{n} {word} moved to the Trash")),
        "restored" => ("notice-success", format!("{n} {word} restored from the Trash")),
        "deleted" => ("notice-success", format!("{n} {word} deleted")),
        "replied" => ("notice-success", "Reply posted".to_string()),
        "edited" => ("notice-success", "Comment updated".to_string()),
        "empty" => ("notice-error", "Comment is empty".to_string()),
        "gone" => ("notice-error", "Comment not found".to_string()),
        _ => return String::new(),
    };
    format!("<div class=\"notice {class}\"><p>{}</p></div>", esc(&text))
}

/// `Y/m/d \a\t g:i a` (WP `get_comment_date`/`get_comment_time` on
/// `created_at`, stored as `YYYY-MM-DD HH:MM:SS` UTC — same display as WP's
/// admin which uses comment_date for the column).
fn submitted_on(created_at: &str) -> String {
    let d = created_at.get(..10).unwrap_or(created_at);
    let (y, m, day) = (d.get(..4).unwrap_or(""), d.get(5..7).unwrap_or(""), d.get(8..10).unwrap_or(""));
    let t = created_at.get(11..16).unwrap_or("00:00");
    let hh: u32 = t.get(..2).and_then(|x| x.parse().ok()).unwrap_or(0);
    let mm = t.get(3..5).unwrap_or("00");
    let ampm = if hh < 12 { "am" } else { "pm" };
    let h12 = hh % 12;
    let h12 = if h12 == 0 { 12 } else { h12 };
    format!("{y}/{m}/{day} at {h12}:{mm} {ampm}")
}

/// Excerpt for the Comment cell — collapsible whitespace, ~300 chars.
fn excerpt(body: &str) -> String {
    let flat: String = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut out: String = flat.chars().take(300).collect();
    if flat.chars().count() > 300 {
        out.push('…');
    }
    esc(&out)
}

/// Per-user `edit-comments.php` Screen Options: which columns show. Persisted
/// as JSON in `settings` under `comment_screen_{user_id}` (no `users.options`
/// column — WP keeps this in usermeta; settings is our equivalent).
#[derive(serde::Serialize, serde::Deserialize)]
struct Cols {
    #[serde(default = "cols_true")]
    author: bool,
    #[serde(default = "cols_true")]
    comment: bool,
    #[serde(default = "cols_true")]
    response: bool,
    #[serde(default = "cols_true")]
    date: bool,
}
fn cols_true() -> bool {
    true
}

impl Cols {
    fn all() -> Self {
        Cols { author: true, comment: true, response: true, date: true }
    }
    /// Missing/corrupt JSON → WP default: every column visible.
    fn load(db: &std::path::Path, user_id: i64) -> Self {
        crud::comments::setting_get(db, &format!("comment_screen_{user_id}"))
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_else(Self::all)
    }
    /// " hidden" suffix for a column's `th`/`td` classes (WP keeps the cell
    /// in the DOM and adds `.hidden`; toggling JS does the same).
    fn cls(&self, on: bool) -> &'static str {
        if on {
            ""
        } else {
            " hidden"
        }
    }
}

/// htmx-triggered request? (`HX-Request: true` header.)
fn is_htmx(headers: &HeaderMap) -> bool {
    headers.get("hx-request").and_then(|v| v.to_str().ok()) == Some("true")
}

/// Strip a `<tr class>` attribute to the parity tokens safe to echo back:
/// alternating/thread/depth only — `comment` and the status token are
/// recomputed server-side so a status flip can't ride the attribute.
fn clean_cls(cls: &str) -> String {
    cls.split_whitespace()
        .filter(|t| {
            matches!(t.as_ref(), "even" | "odd" | "alt" | "thread-even" | "thread-odd" | "thread-alt")
                || t.starts_with("depth-")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// One `<tr>` for `tbody#the-list` — shared by the full list and the htmx
/// fragments (quick-edit save, reply save). `cls` carries only the parity
/// tokens (even/odd/thread-*/depth-*); `comment` + status are appended here.
fn row_html(sc: &SiteCtx, r: &Cmt, cols: &Cols, cls: &str) -> String {
    let cb = format!(
        "<th scope=\"row\" class=\"check-column\"><input form=\"comments-bulk\" type=\"checkbox\" name=\"p-{}\"></th>",
        r.id
    );
    let author = author_cell(r);
    let comment = format!("{}{}", excerpt(&r.body), row_acts(r));
    let response = response_cell(sc, r);
    let date = submitted_on(&r.created_at);
    let row_class = format!(
        "comment {} {}",
        cls,
        match r.status.as_str() {
            "pending" => "unapproved",
            "spam" => "spam",
            "trash" | "delete" => "trash",
            _ => "approved",
        }
    );
    format!(
        "<tr id=\"comment-{id}\" class=\"{row_class}\" data-author=\"{author_attr}\" data-email=\"{email_attr}\" data-url=\"{url_attr}\" data-body=\"{body_attr}\">\
         {cb}<td class=\"author column-author{h_author}\">{author}</td>\
         <td class=\"comment column-comment column-primary{h_comment}\">{comment}</td>\
         <td class=\"response column-response{h_response}\">{response}</td>\
         <td class=\"date column-date{h_date}\">{date}</td></tr>",
        id = r.id,
        row_class = row_class,
        h_author = cols.cls(cols.author),
        h_comment = cols.cls(cols.comment),
        h_response = cols.cls(cols.response),
        h_date = cols.cls(cols.date),
        author_attr = esc_attr(&r.author),
        email_attr = esc_attr(&r.email),
        url_attr = esc_attr(&r.author_url),
        body_attr = esc_attr(&r.body),
    )
}

/// WP `#screen-meta` + `#screen-meta-links`: the "Screen Options" tab and its
/// collapsible panel of `{col}-hide` checkboxes (checked = column visible).
fn screen_options_html(cols: &Cols, csrf: &str, back: &str) -> String {
    let cb = |name: &str, on: bool, label: &str| {
        format!(
            "<label for=\"{name}-hide\"><input class=\"hide-column-tog\" name=\"{name}-hide\" type=\"checkbox\" id=\"{name}-hide\" value=\"{name}\"{}>{label}</label>",
            if on { " checked" } else { "" }
        )
    };
    format!(
        r#"<div id="screen-meta" class="metabox-prefs">
<div id="screen-options-wrap" class="hidden" tabindex="-1" aria-label="Screen Options Tab">
<form id="adv-settings" method="post" action="/admin/comments/screen">
<input type="hidden" name="csrf" value="{csrf}">
<input type="hidden" name="back" value="{back}">
<fieldset class="metabox-prefs view-options">
<legend>Columns</legend>
{cb_author}
{cb_comment}
{cb_response}
{cb_date}
</fieldset>
<p class="submit"><input type="submit" class="button button-primary" id="screen-options-apply" value="Apply"></p>
</form>
</div>
</div>
<div id="screen-meta-links">
<div id="screen-options-link-wrap" class="hide-if-no-js screen-meta-toggle">
<button type="button" id="show-settings-link" class="button show-settings" aria-controls="screen-options-wrap" aria-expanded="false">Screen Options</button>
</div>
</div>"#,
        csrf = csrf,
        back = esc(back),
        cb_author = cb("author", cols.author, "Author"),
        cb_comment = cb("comment", cols.comment, "Comment"),
        cb_response = cb("response", cols.response, "In Response To"),
        cb_date = cb("date", cols.date, "Submitted On"),
    )
}

/// `.row-actions` spans for one row, per WP's status-dependent action set.
/// Link actions (Reply/Quick Edit/Edit) can't use `list::row_actions`
/// (POST-forms only), so this emits both kinds in one helper.
fn row_acts(r: &Cmt) -> String {
    let mut parts: Vec<String> = Vec::new();
    // Edit expands an inline `<tr>` fragment (htmx `afterend`), same
    // mechanism as Reply; plain href keeps the standalone comment.php page
    // as the no-JS fallback.
    let edit = format!(
        "<span class=\"edit\"><a href=\"/admin/comments/edit/{id}\" class=\"edit-expand\" \
         hx-get=\"/admin/comments/edit/{id}?frag=1\" hx-target=\"closest tr\" hx-swap=\"afterend\">Edit</a></span>",
        id = r.id,
    );
    let btn = |class: &str, label: &str, verb: &str, confirm: &str| -> String {
        let oc = if confirm.is_empty() {
            String::new()
        } else {
            format!(" onclick=\"return confirm('{}')\"", confirm)
        };
        format!(
            "<span class=\"{class}\"><button type=\"submit\" class=\"button-link\" \
             form=\"cmt-act\" formaction=\"/admin/comments/action?action={verb}&amp;id={id}\"{oc}>{label}</button></span>",
            id = r.id,
        )
    };
    // WP's inline `.reply-row`: htmx fetches the form <tr> and inserts it
    // after this row; `cls` (sanitized server-side) keeps the reply form's
    // own parity tokens for the eventual replacement row. No-JS falls back
    // to the standalone reply page via the href.
    let reply = format!(
        "<span class=\"reply hide-if-no-js\"><a href=\"/admin/comments/reply/{id}\" \
         hx-get=\"/admin/comments/reply/{id}\" hx-target=\"closest tr\" hx-swap=\"afterend\" \
         hx-vals='js:{{cls: this.closest(\"tr\").className}}'>Reply</a></span>",
        id = r.id,
    );
    let quick_edit = format!(
        "<span class=\"inline hide-if-no-js\"><a href=\"#{}\" class=\"editinline\" data-id=\"{}\">Quick Edit</a></span>",
        r.id, r.id
    );
    match r.status.as_str() {
        "spam" => {
            parts.push(btn("unspam", "Not Spam", "notspam", ""));
            parts.push(btn("delete", "Delete Permanently", "delete", "Permanently delete this comment?"));
        }
        "trash" => {
            parts.push(btn("restore", "Restore", "restore", ""));
            parts.push(btn("delete", "Delete Permanently", "delete", "Permanently delete this comment?"));
        }
        "pending" => {
            parts.push(btn("approve", "Approve", "approve", ""));
            parts.push(reply.clone());
            parts.push(quick_edit);
            parts.push(edit.clone());
            parts.push(btn("spam", "Spam", "spam", ""));
            parts.push(btn("trash", "Trash", "trash", ""));
        }
        _ => {
            // approved (and anything unexpected in the All view)
            parts.push(btn("unapprove", "Unapprove", "unapprove", ""));
            parts.push(reply.clone());
            parts.push(quick_edit);
            parts.push(edit);
            parts.push(btn("spam", "Spam", "spam", ""));
            parts.push(btn("trash", "Trash", "trash", ""));
        }
    }
    format!("<div class=\"row-actions\">{}</div>", parts.join(" | "))
}

/// Stacked author cell: name, URL link, email link, IP (WP `column_author`).
fn author_cell(r: &Cmt) -> String {
    let mut out = format!("<strong>{}</strong><br>", esc(&r.author));
    if !r.author_url.is_empty() {
        let u = &r.author_url;
        if u.starts_with("http://") || u.starts_with("https://") || u.starts_with('/') {
            out.push_str(&format!("<a href=\"{}\">{}</a><br>", esc(u), esc(u)));
        } else {
            // Non-web scheme (javascript:, data:, …) → text only, no href.
            out.push_str(&format!("{}<br>", esc(u)));
        }
    }
    if !r.email.is_empty() {
        out.push_str(&format!("<a href=\"mailto:{}\">{}</a><br>", esc(&r.email), esc(&r.email)));
    }
    if !r.author_ip.is_empty() {
        out.push_str(&esc(&r.author_ip));
    }
    out
}

/// In Response To: post title → this list filtered by post_id, plus
/// "View Post" and the post's approved-comment bubble.
fn response_cell(sc: &SiteCtx, r: &Cmt) -> String {
    let Some(pid) = r.post_id else {
        return "—".to_string();
    };
    let title = if r.post_title.is_empty() { "(no title)" } else { &r.post_title };
    let view = posts::public_path(sc, &r.post_lang, &r.post_kind, &r.post_slug);
    // WP shows: post title link | pending-count bubble | "View Post" + line below
    // "<n> approved comments | <m> pending comments | <k> total comments".
    // We have post_comments (approved) only; pending/total derive lazily via a
    // lookup query — keep counts light: hide subtext when zero.
    let pend = r.post_comments_pending;
    let mut sub = String::new();
    if r.post_comments == 0 {
        sub.push_str("<span class=\"post-com-count post-com-count-no-comments\"><span class=\"comment-count comment-count-no-comments\">0</span><span class=\"screen-reader-text\">No comments</span></span>");
        sub.push_str("<span class=\"post-com-count-text\">No approved comments</span>");
    }
    if pend > 0 {
        sub.push_str(&format!(" <span class=\"post-com-count post-com-count-pending\"><span class=\"comment-count comment-count-pending\">{pend}</span></span><span class=\"post-com-count-pending-text\"> {pend} pending comments</span>"));
    }
    format!(
        "<a href=\"/admin/comments?post_id={pid}\">{}</a><br>\
         <div class=\"response-links\"><span class=\"post-com-count-wrapper\"><a href=\"/admin/comments?post_id={pid}\" class=\"post-com-count post-com-count-approved\"><span class=\"comment-count-approved\" aria-hidden=\"true\">{}</span></a></span>\
         <a class=\"comments-view-item-link\" href=\"{}\" target=\"_blank\">View Post</a></div>{sub}",
        esc(title), r.post_comments, esc(&view),
    )
}
pub(crate) async fn comments_page(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    let view = q.get("status").map(|s| s.as_str()).unwrap_or("all");
    let view = match view {
        "mine" | "pending" | "approved" | "spam" | "trash" => view,
        _ => "all",
    };
    let post_id = q.get("post_id").and_then(|v| v.parse::<i64>().ok());
    let search = q.get("s").map(|s| s.as_str()).unwrap_or("");
    let counts = crud::comments::counts_for(&sc.db, &s.email).unwrap_or_default();
    let filter = crud::comments::Filter {
        status: view,
        post_id,
        s: search,
        // "Mine" narrows to comments authored under the session email.
        by_author_email: if view == "mine" { &s.email } else { "" },
        limit: list::PER_PAGE,
        offset: (q.get("paged").and_then(|v| v.parse::<i64>().ok()).unwrap_or(1).max(1) - 1) * list::PER_PAGE,
    };
    let Ok((rows, total)) = crud::comments::list(&sc.db, &filter) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let pager = list::Pager::new(&q, total);
    let back = back_url(&q);

    let cols = Cols::load(&sc.db, s.user_id);

    // subsubsub — zero-count views hidden (list::views_bar); Mine sits
    // between All and Pending like WP's edit-comments.php.
    let mut extra: HashMap<String, String> = HashMap::new();
    if let Some(pid) = post_id {
        extra.insert("post_id".into(), pid.to_string());
    }
    if !search.is_empty() {
        extra.insert("s".into(), search.to_string());
    }
    let views = list::views_bar(
        "/admin/comments?",
        "status",
        view,
        &[
            list::View { key: "all", label: "All", count: counts.all },
            list::View { key: "mine", label: "Mine", count: counts.mine },
            list::View { key: "pending", label: "Pending", count: counts.pending },
            list::View { key: "approved", label: "Approved", count: counts.approved },
            list::View { key: "spam", label: "Spam", count: counts.spam },
            list::View { key: "trash", label: "Trash", count: counts.trash },
        ],
        &extra,
    );

    // Pager preserves the current view + filters.
    let mut pextra: HashMap<String, String> = extra.clone();
    if view != "all" {
        pextra.insert("status".into(), view.to_string());
    }

    // Rows — shared `row_html` so htmx fragment responses render identically.
    let mut body_rows = String::new();
    for (i, r) in rows.iter().enumerate() {
        let (odd, thread) = if i % 2 == 0 { ("even", "thread-even") } else { ("odd alt", "thread-odd thread-alt") };
        let depth = if r.parent_id.is_some() { 2 } else { 1 };
        body_rows.push_str(&row_html(&sc, r, &cols, &format!("{odd} {thread} depth-{depth}")));
    }
    if body_rows.is_empty() {
        let msg = if search.is_empty() { "No comments found." } else { "No comments found matching your search." };
        body_rows.push_str(&list::empty_row(5, msg));
    }

    // Bulk actions by view (WP labels).
    let bulk: &[(&str, &str)] = match view {
        "pending" => &[("approve", "Approve"), ("spam", "Mark as Spam"), ("trash", "Move to Trash")],
        "approved" => &[("unapprove", "Unapprove"), ("spam", "Mark as Spam"), ("trash", "Move to Trash")],
        "spam" => &[("notspam", "Not Spam"), ("delete", "Delete Permanently")],
        "trash" => &[("restore", "Restore"), ("delete", "Delete Permanently")],
        _ => &[
            ("approve", "Approve"),
            ("unapprove", "Unapprove"),
            ("spam", "Mark as Spam"),
            ("trash", "Move to Trash"),
        ],
    };
    // Top nav: same chrome as list::tablenav but built inline so WP's
    // comment-type <select> can sit next to the bulk actions. The select is
    // a parity visual only — comments have no type column (no pingbacks/
    // trackbacks), so Filter is a no-op; fields ride `form="comment-filter"`,
    // a GET form outside #comments-bulk (forms can't nest).
    let topnav = {
        let mut t = String::from("<div class=\"tablenav top\"><div class=\"actions bulkactions\">\
            <label class=\"screen-reader-text\">Select bulk action</label><select name=\"action\">\
            <option value=\"-1\">Bulk actions</option>");
        for (v, label) in bulk {
            t.push_str(&format!("<option value=\"{v}\">{label}</option>"));
        }
        t.push_str(&format!(
            "</select><input type=\"submit\" class=\"button action\" value=\"Apply\"></div>\
             <div class=\"alignleft actions\">\
             <label for=\"filter-by-comment-type\" class=\"screen-reader-text\">Filter by comment type</label>\
             <select form=\"comment-filter\" id=\"filter-by-comment-type\" name=\"comment_type\">\
             <option value=\"\">All comment types</option>\
             <option value=\"comment\">Comments</option></select>\
             <input form=\"comment-filter\" type=\"submit\" class=\"button\" value=\"Filter\"></div>{}<br class=\"clear\"></div>",
            pager.nav("/admin/comments", &pextra)
        ));
        t
    };
    // Bottom nav mirrors top but uses WP's action2 select (top select can't
    // shadow it through serde_urlencoded's last-wins collapse).
    let empty_btn = match view {
        "spam" => "<button type=\"submit\" class=\"button\" form=\"comments-bulk\" formaction=\"/admin/comments/empty?status=spam\">Empty Spam</button>",
        "trash" => "<button type=\"submit\" class=\"button\" form=\"comments-bulk\" formaction=\"/admin/comments/empty?status=trash\">Empty Trash</button>",
        _ => "",
    };
    let bottomnav = {
        let mut b = String::from("<div class=\"tablenav bottom\"><div class=\"actions bulkactions\">\
            <label class=\"screen-reader-text\">Select bulk action</label><select form=\"comments-bulk\" name=\"action2\">\
            <option value=\"-1\">Bulk actions</option>");
        for (v, label) in bulk {
            b.push_str(&format!("<option value=\"{v}\">{label}</option>"));
        }
        b.push_str(&format!(
            "</select><input form=\"comments-bulk\" type=\"submit\" class=\"button action\" value=\"Apply\"></div>{empty_btn}<br class=\"clear\"></div>"
        ));
        b
    };

    let search_box = format!(
        r#"<form method="get" action="/admin/comments">
        <p class="search-box"><label class="screen-reader-text" for="comment-search-input">Search Comments:</label>
        <input type="search" id="comment-search-input" name="s" value="{}">
        <input type="hidden" name="status" value="{view}">
        <input type="submit" id="search-submit" class="button" value="Search Comments"></p></form>"#,
        esc(search)
    );

    // Quick Edit inline row — posts.rs pattern: out-of-table form + hidden
    // <tr> relocated by JS. Editable: author, email, URL, body (not status).
    let qe_row = format!(
        r##"<tr id="qe-row" class="inline-edit-row quick-edit-row" style="display:none"><td colspan="5" class="colspanchange">
<input form="qe-cmt-form" type="hidden" name="id" value="">
<input form="qe-cmt-form" type="hidden" name="cls" id="qe-cls" value="">
<fieldset><legend class="inline-edit-legend">Quick Edit</legend>
<div class="inline-edit-col">
<label class="inline-edit-group"><span class="title">Name</span><input form="qe-cmt-form" type="text" name="author" id="qe-author" style="width:360px"></label>
<label class="inline-edit-group"><span class="title">Email</span><input form="qe-cmt-form" type="text" name="email" id="qe-email" style="width:360px"></label>
<label class="inline-edit-group"><span class="title">URL</span><input form="qe-cmt-form" type="text" name="url" id="qe-url" style="width:360px"></label>
<label class="inline-edit-group"><span class="title">Comment</span><textarea form="qe-cmt-form" name="body" id="qe-body" rows="5" cols="50"></textarea></label>
</div>
<p class="submit inline-edit-save" style="margin-top:10px">
<button form="qe-cmt-form" type="submit" class="button button-primary" id="qe-update" hx-post="/admin/comments/quickedit" hx-include="#qe-cmt-form" hx-swap="outerHTML">Update Comment</button>
<button type="button" class="button" id="qe-cancel">Cancel</button>
</p>
</fieldset>
</td></tr>"##
    );

    let notice = notice_html(&q);
    let head = list::thead(&[
        (format!("author{}", cols.cls(cols.author)).as_str(), "Author", ""),
        (format!("comment column-primary{}", cols.cls(cols.comment)).as_str(), "Comment", ""),
        (format!("response{}", cols.cls(cols.response)).as_str(), "In Response To", ""),
        (format!("date{}", cols.cls(cols.date)).as_str(), "Submitted On", ""),
    ]);

    let inner = format!(
        r##"{chrome}{screen_meta}<style>
#screen-meta-links{{float:right;margin:0 20px 0 0}}
#screen-meta-links .show-settings{{border:1px solid #c3c4c7;border-top:0;background:#f6f7f7;padding:3px 12px;border-radius:0 0 4px 4px}}
#screen-options-wrap{{background:#fff;border:1px solid #c3c4c7;padding:15px 20px;box-shadow:0 2px 5px rgba(0,0,0,.15)}}
#screen-options-wrap.hidden,.wp-list-table .hidden{{display:none}}
.reply-row td{{padding:12px}}
</style><div class="wrap"><h1 class="wp-heading-inline">Comments</h1>
<hr class="wp-header-end">
{notice}{views}
{search_box}
<form method="post" action="/admin/comments/bulk" id="comments-bulk"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="back" value="{back}">
{topnav}
<table class="wp-list-table widefat fixed striped table-view-list comments"><thead>{head}</thead><tbody id="the-list">{body_rows}{qe_row}</tbody></table>
{bottomnav}
</form>
<form method="get" action="/admin/comments" id="comment-filter"><input type="hidden" name="status" value="{view}"><input type="hidden" name="post_id" value="{post_id_v}"><input type="hidden" name="s" value="{search_v}"></form>
<form method="post" action="/admin/comments/reply" id="cmt-reply"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="back" value="{back}"></form>
<form method="post" action="/admin/comments/action" id="cmt-act"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="back" value="{back}"></form>
<form method="post" action="/admin/comments/quickedit" id="qe-cmt-form"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="back" value="{back}"></form>
<form method="post" action="/admin/comments/save" id="cmt-edit"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="back" value="{back}"></form>
</div>
<script>
(function(){{
var qe=document.getElementById('qe-row');
document.addEventListener('click',function(e){{
  var a=e.target.closest('a.editinline');
  if(a&&qe){{
    e.preventDefault();
    var tr=a.closest('tr');var id=a.getAttribute('data-id');
    qe.querySelector('input[name=id]').value=id;
    qe.querySelector('#qe-cls').value=tr.className;
    qe.querySelector('#qe-author').value=tr.getAttribute('data-author')||'';
    qe.querySelector('#qe-email').value=tr.getAttribute('data-email')||'';
    qe.querySelector('#qe-url').value=tr.getAttribute('data-url')||'';
    qe.querySelector('#qe-body').value=tr.getAttribute('data-body')||'';
    var up=document.getElementById('qe-update');
    if(up)up.setAttribute('hx-target','#comment-'+id);
    tr.parentNode.insertBefore(qe,tr.nextSibling);
    qe.style.display='';
    return;
  }}
  // One inline edit row at a time: a second open fragment would add
  // another `name=id` field to the shared #cmt-edit form.
  if(e.target&&e.target.closest&&e.target.closest('a.edit-expand')){{
    var open=document.querySelector('tr.comment-edit-row');
    if(open)open.parentNode.removeChild(open);
  }}
  if(qe&&e.target&&e.target.id==='qe-cancel'){{
    qe.style.display='none';
  }}
  if(e.target&&e.target.id==='show-settings-link'){{
    var w=document.getElementById('screen-options-wrap');
    var hid=w.classList.toggle('hidden');
    e.target.setAttribute('aria-expanded',hid?'false':'true');
  }}
}});
// htmx swapped the updated <tr> in over the original — drop the editor row.
if(qe){{
  document.addEventListener('htmx:afterSwap',function(){{
    qe.style.display='none';
  }});
}}
// WP hide-column-tog behavior: toggle .hidden on every .column-* cell live;
// persistence happens via the Apply POST to /admin/comments/screen.
document.addEventListener('change',function(e){{
  var t=e.target;
  if(!t||!t.classList||!t.classList.contains('hide-column-tog'))return;
  var cells=document.querySelectorAll('.column-'+t.value);
  for(var i=0;i<cells.length;i++)cells[i].classList.toggle('hidden',!t.checked);
}});
}})();
</script>"##,
        chrome = chrome("comments", &s),
        screen_meta = screen_options_html(&cols, &s.csrf, &back),
        csrf = s.csrf,
        back = esc(&back),
        post_id_v = post_id.map(|v| v.to_string()).unwrap_or_default(),
        search_v = esc(search),
        view = view,
    );
    html_admin(&s, "Comments", &inner, true, "edit-comments-php").into_response()
}

/// verb → (new status, notice key). Row action + bulk share this map.
fn status_target(verb: &str) -> Option<(&'static str, &'static str)> {
    match verb {
        "approve" => Some(("approved", "approved")),
        "unapprove" => Some(("pending", "unapproved")),
        "spam" => Some(("spam", "spammed")),
        "notspam" => Some(("pending", "unspammed")),
        "trash" => Some(("trash", "trashed")),
        "restore" => Some(("pending", "restored")),
        _ => None,
    }
}

/// POST /admin/comments/action — id+action arrive as form fields or as
/// `?action=…&id=…` on the formaction URL; back/csrf always form fields.
pub(crate) async fn comment_action(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if f.get("csrf").map(|x| x.as_str()) != Some(s.csrf.as_str()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let back = list::back_or(&f, "/admin/comments");
    let action = f
        .get("action")
        .or_else(|| q.get("action"))
        .map(|s| s.as_str())
        .unwrap_or("");
    let id = f
        .get("id")
        .or_else(|| q.get("id"))
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    if id == 0 {
        return Redirect::to(back).into_response();
    }
    // Approve/unapprove/spam/trash/delete all change what the post page
    // renders → queue the single-file regen (phase 4.4). The post id comes
    // from the row itself (before the delete path drops it).
    let post_id = crud::comments::get(&sc.db, id)
        .ok()
        .flatten()
        .and_then(|r| r.post_id)
        .unwrap_or(0);
    match action {
        "delete" => {
            let _ = crud::comments::delete_permanent(&sc.db, id);
            admin::regen_posts(sc, &[post_id]);
            redirect_back(back, "deleted", 1)
        }
        _ => match status_target(action) {
            Some((st, m)) => {
                let _ = crud::comments::set_status(&sc.db, id, st);
                admin::regen_posts(sc, &[post_id]);
                redirect_back(back, m, 1)
            }
            None => Redirect::to(back).into_response(),
        },
    }
}

/// POST /admin/comments/bulk — `p-{id}` checkboxes + action/action2 (WP's
/// top/bottom bulk select names; HashMap collapse makes last-wins, so a
/// top-level `action=-1` falls through to `action2` server-side).
pub(crate) async fn comments_bulk(
    State(app): State<App>,
    headers: HeaderMap,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if f.get("csrf").map(|x| x.as_str()) != Some(s.csrf.as_str()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let back = list::back_or(&f, "/admin/comments");
    let mut action = f.get("action").map(|s| s.as_str()).unwrap_or("-1");
    if action == "-1" {
        action = f.get("action2").map(|s| s.as_str()).unwrap_or("-1");
    }
    let mut ids: Vec<i64> = f
        .keys()
        .filter_map(|k| k.strip_prefix("p-").and_then(|x| x.parse::<i64>().ok()))
        .collect();
    ids.sort_unstable();
    ids.dedup();
    let mut done = 0i64;
    // One regen per affected post after the mutations (phase 4.4).
    let mut posts: Vec<i64> = Vec::new();
    match action {
        "delete" => {
            for id in &ids {
                if let Ok(Some(r)) = crud::comments::get(&sc.db, *id) {
                    if let Some(pid) = r.post_id {
                        posts.push(pid);
                    }
                }
                if crud::comments::delete_permanent(&sc.db, *id).is_ok() {
                    done += 1;
                }
            }
        }
        _ => {
            if let Some((st, _)) = status_target(action) {
                for id in &ids {
                    if let Ok(Some(r)) = crud::comments::get(&sc.db, *id) {
                        if let Some(pid) = r.post_id {
                            posts.push(pid);
                        }
                    }
                    if crud::comments::set_status(&sc.db, *id, st).is_ok() {
                        done += 1;
                    }
                }
            }
        }
    }
    if done > 0 {
        admin::regen_posts(sc, &posts);
    }
    let m = if action == "delete" { "deleted" } else { status_target(action).map(|x| x.1).unwrap_or("edited") };
    if done == 0 {
        return Redirect::to(back).into_response();
    }
    redirect_back(back, m, done)
}

/// POST /admin/comments/empty?status=spam|trash — Empty Spam / Empty Trash.
pub(crate) async fn comments_empty(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if f.get("csrf").map(|x| x.as_str()) != Some(s.csrf.as_str()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let back = list::back_or(&f, "/admin/comments");
    let status = q
        .get("status")
        .or_else(|| f.get("status"))
        .map(|s| s.as_str())
        .unwrap_or("");
    let n = match status {
        "spam" | "trash" => crud::comments::empty_status(&sc.db, status).unwrap_or(0) as i64,
        _ => 0,
    };
    redirect_back(back, "deleted", n)
}

/// `aa-mm-jj @ hh:mm` timestamp editor (WP `comment.php` Submitted On block).
/// `form_attr` is either "" (fields inside a real <form>) or ` form="…"`
/// so inline-fragment inputs can ride an out-of-table form.
fn date_fields(created_at: &str, form_attr: &str) -> String {
    let d = created_at.get(..10).unwrap_or("2000-01-01");
    let t = created_at.get(11..16).unwrap_or("00:00");
    let (aa, mm, jj) = (d.get(..4).unwrap_or("2000"), d.get(5..7).unwrap_or("01"), d.get(8..10).unwrap_or("01"));
    let (hh, mn) = (t.get(..2).unwrap_or("00"), t.get(3..5).unwrap_or("00"));
    let mut opts = String::new();
    for (v, name) in [
        ("01", "Jan"), ("02", "Feb"), ("03", "Mar"), ("04", "Apr"), ("05", "May"), ("06", "Jun"),
        ("07", "Jul"), ("08", "Aug"), ("09", "Sep"), ("10", "Oct"), ("11", "Nov"), ("12", "Dec"),
    ] {
        let sel = if v == mm { " selected" } else { "" };
        opts.push_str(&format!("<option value=\"{v}\"{sel}>{v}-{name}</option>"));
    }
    format!(
        r#"@ <input{fa} type="text" name="hh" value="{hh}" size="2" maxlength="2" autocomplete="off"> :
        <input{fa} type="text" name="mn" value="{mn}" size="2" maxlength="2"><br>
        <input{fa} type="text" name="jj" value="{jj}" size="2" maxlength="2">
        <select{fa} name="mm">{opts}</select>
        <input{fa} type="text" name="aa" value="{aa}" size="4" maxlength="4">"#,
        fa = form_attr,
    )
}

/// Edit Comment screen (WP comment.php): Name, Email, URL, body, status
/// radios (approved/pending/spam — trash isn't a settable radio), editable
/// Submitted On.
pub(crate) async fn edit_page(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    let Ok(Some(r)) = crud::comments::get(&sc.db, id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    // `?frag=1` (htmx row-action) → just the inline editor <tr>, inserted
    // `afterend` the comment row. Fields ride the out-of-table #cmt-edit
    // form via `form=` like Reply/Quick Edit; #cmt-edit already carries
    // csrf + back so Update lands on the filtered list.
    if q.get("frag").map(|v| v.as_str()) == Some("1") {
        let frag = format!(
            r##"<tr class="inline-edit-row comment-edit-row" style="display:table-row"><td colspan="5" class="colspanchange">
<input form="cmt-edit" type="hidden" name="id" value="{id}">
<fieldset><legend class="inline-edit-legend">Edit Comment</legend>
<div class="inline-edit-col">
<label class="inline-edit-group"><span class="title">Name</span><input form="cmt-edit" type="text" name="author" value="{author}" style="width:360px"></label>
<label class="inline-edit-group"><span class="title">Email</span><input form="cmt-edit" type="text" name="email" value="{email}" style="width:360px"></label>
<label class="inline-edit-group"><span class="title">URL</span><input form="cmt-edit" type="text" name="url" value="{url}" style="width:360px"></label>
<label class="inline-edit-group"><span class="title">Comment</span><textarea form="cmt-edit" name="body" rows="5" cols="50">{body}</textarea></label>
<label class="inline-edit-group"><span class="title">Submitted On</span><span>{date}</span></label>
<div class="inline-edit-group"><span class="title">Status</span>
<label><input form="cmt-edit" type="radio" name="status" value="approved"{st_a_ck}> Approved</label>
<label><input form="cmt-edit" type="radio" name="status" value="pending"{st_p_ck}> Pending</label>
<label><input form="cmt-edit" type="radio" name="status" value="spam"{st_s_ck}> Spam</label>
</div>
</div>
<p class="submit inline-edit-save" style="margin-top:10px">
<button form="cmt-edit" type="submit" class="button button-primary">Update</button>
<button type="button" class="button" onclick="this.closest('tr').remove()">Cancel</button>
</p>
</fieldset>
</td></tr>"##,
            id = r.id,
            author = esc(&r.author),
            email = esc(&r.email),
            url = esc(&r.author_url),
            body = esc(&r.body),
            date = date_fields(&r.created_at, " form=\"cmt-edit\""),
            st_a_ck = if r.status == "approved" { " checked" } else { "" },
            st_p_ck = if r.status == "pending" { " checked" } else { "" },
            st_s_ck = if r.status == "spam" { " checked" } else { "" },
        );
        return Html(frag).into_response();
    }
    // Post title for the "In response to" sidebar line.
    let mut in_response = "<em>Orphaned</em>".to_string();
    if let Some(pid) = r.post_id {
        if let Ok(Some(p)) = crud::load_post(&sc.db, pid) {
            let link = posts::public_path(&sc, &p.lang, &p.kind, &p.slug);
            let t = if p.title.is_empty() { "(no title)" } else { p.title.as_str() };
            in_response = format!("<strong><a href=\"{}\" target=\"_blank\">{}</a></strong>", esc(&link), esc(t));
        }
    }
    let in_reply = if let Some(pp) = r.parent_id {
        let t = crud::comments::get(&sc.db, pp).ok().flatten().map(|p| p.author).unwrap_or_else(|| "#".into());
        format!("{} <a href=\"/admin/comments/edit/{pp}\">Edit</a>", esc(&t))
    } else {
        "None".to_string()
    };
    let status_cur = match r.status.as_str() {
        "approved" => "Approved",
        "spam" => "Spam",
        "trash" | "delete" => "Trash",
        _ => "Pending",
    };
    let notice = notice_html(&q);
    let inner = format!(
        r##"{chrome}<div class="wrap"><h1 class="wp-heading-inline">Edit Comment</h1><hr class="wp-header-end">
{notice}
<div id="poststuff"><form method="post" action="/admin/comments/save">
<input type="hidden" name="csrf" value="{csrf}">
<input type="hidden" name="id" value="{id}">
<input type="hidden" name="back" value="/admin/comments">
<div id="post-body" class="metabox-holder columns-2">
<div id="side-info-column" class="inner-sidebar">
<div id="submitdiv" class="stuffbox"><h3 class="hndle">Save</h3><div class="inside">
<div class="submitbox" id="submitcomment">
<div id="misc-publishing-actions">
<div class="misc-pub-section misc-pub-comment-status" id="comment-status"><label for="comment_status">Status:</label>
<span id="comment-status-display"> {status_cur}</span>
<div id="comment-status-select">
<input type="radio" name="status" value="approved" id="comment-status-approved"{st_a_ck}> <label for="comment-status-approved">Approved</label><br>
<input type="radio" name="status" value="pending" id="comment-status-pending"{st_p_ck}> <label for="comment-status-pending">Pending</label><br>
<input type="radio" name="status" value="spam" id="comment-status-spam"{st_s_ck}> <label for="comment-status-spam">Spam</label>
</div></div>
<div class="misc-pub-section curtime misc-pub-curtime"><span id="timestamp">Submitted on: <b>{submitted}</b></span>
<div id="timestampdiv">{date}</div></div>
<div class="misc-pub-section misc-pub-response-to">In response to: {in_response}</div>
<div class="misc-pub-section misc-pub-reply-to">In reply to: {in_reply}</div>
</div>
<div id="major-publishing-actions">
<div id="delete-action"><button type="submit" name="action" value="trash" class="submitdelete deletion" form="cmt-act" formaction="/admin/comments/action?action=trash&amp;id={id}">Move to Trash</button></div>
<div id="publishing-action"><button type="submit" class="button button-primary button-large" id="publish" name="save">Update</button></div>
<div class="clear"></div>
</div>
</div></div></div>
</div>
<div id="post-body-content">
<div id="namediv" class="stuffbox"><h3><label for="name">Author</label></h3><div class="inside">
<input type="text" name="author" id="name" value="{author}" size="30" class="regular-text">
<table class="form-table editcomment">
<tr><td class="first"><label for="email">Email</label></td><td><input type="text" name="email" id="email" value="{email}" size="30" class="regular-text"></td></tr>
<tr><td class="first"><label for="newcommentauthorurl">URL</label></td><td><input type="text" name="url" id="newcommentauthorurl" value="{url}" size="30" class="regular-text code"></td></tr>
</table>
</div></div>
<div class="stuffbox"><h3><label for="body">Comment</label></h3><div class="inside">
<div class="quicktags-toolbar" id="qt_content_toolbar">
<input type="button" class="ed_button button button-small" value="b">
<input type="button" class="ed_button button button-small" value="i" style="font-style:italic">
<input type="button" class="ed_button button button-small" value="link">
<input type="button" class="ed_button button button-small" value="b-quote">
<input type="button" class="ed_button button button-small" value="del">
<input type="button" class="ed_button button button-small" value="ins">
<input type="button" class="ed_button button button-small" value="img">
<input type="button" class="ed_button button button-small" value="ul">
<input type="button" class="ed_button button button-small" value="ol">
<input type="button" class="ed_button button button-small" value="li">
<input type="button" class="ed_button button button-small" value="code">
<input type="button" class="ed_button button button-small" value="close tags">
</div>
<textarea class="widefat" rows="10" cols="40" name="body" id="body">{body}</textarea>
</div></div>
<div class="clear"></div>
</div>
<div class="clear"></div>
</form></div></div>"##,
        chrome = chrome("comments", &s),
        csrf = s.csrf,
        id = r.id,
        author = esc(&r.author),
        email = esc(&r.email),
        url = esc(&r.author_url),
        body = esc(&r.body),
        status_cur = status_cur,
        st_a_ck = if r.status == "approved" { " checked" } else { "" },
        st_p_ck = if r.status == "pending" { " checked" } else { "" },
        st_s_ck = if r.status == "spam" { " checked" } else { "" },
        submitted = esc(&submitted_on(&r.created_at)),
        date = date_fields(&r.created_at, ""),
        in_response = in_response,
        in_reply = in_reply,
    );
    html_admin(&s, "Edit Comment", &inner, false, "comment-php").into_response()
}

/// POST /admin/comments/save — full edit; empty body bounces with the WP
/// "Comment is empty" error.
pub(crate) async fn edit_save(
    State(app): State<App>,
    headers: HeaderMap,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if f.get("csrf").map(|x| x.as_str()) != Some(s.csrf.as_str()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let back = list::back_or(&f, "/admin/comments");
    let id = f.get("id").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
    if id == 0 {
        return Redirect::to(back).into_response();
    }
    let body = f.get("body").cloned().unwrap_or_default();
    if body.trim().is_empty() {
        return Redirect::to(&format!("/admin/comments/edit/{id}?m=empty")).into_response();
    }
    let status = match f.get("status").map(|s| s.as_str()) {
        Some("pending") => "pending",
        Some("spam") => "spam",
        _ => "approved",
    };
    let get = |k: &str| f.get(k).cloned().unwrap_or_default();
    let (aa, mm, jj, hh, mn) = (get("aa"), get("mm"), get("jj"), get("hh"), get("mn"));
    let created = if [aa.as_str(), mm.as_str(), jj.as_str(), hh.as_str(), mn.as_str()]
        .iter()
        .all(|x| !x.is_empty())
    {
        Some(format!("{aa}-{mm}-{jj} {hh}:{mn}:00"))
    } else {
        None
    };
    let _ = crud::comments::update_fields(
        &sc.db,
        id,
        &get("author"),
        &get("email"),
        &get("url"),
        &body,
        status,
        created.as_deref(),
    );
    let post_id = crud::comments::get(&sc.db, id)
        .ok()
        .flatten()
        .and_then(|r| r.post_id)
        .unwrap_or(0);
    admin::regen_posts(sc, &[post_id]);
    redirect_back(back, "edited", 1)
}

/// POST /admin/comments/quickedit — author/email/url/body only; status and
/// date untouched (WP inline-edit-comment parity). htmx requests get the
/// updated `<tr>` back for an `outerHTML` swap; plain POSTs redirect to the
/// list like before.
pub(crate) async fn quick_edit(
    State(app): State<App>,
    headers: HeaderMap,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if f.get("csrf").map(|x| x.as_str()) != Some(s.csrf.as_str()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let htmx = is_htmx(&headers);
    let back = list::back_or(&f, "/admin/comments");
    let id = f.get("id").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
    let body = f.get("body").cloned().unwrap_or_default();
    if id == 0 || body.trim().is_empty() {
        if htmx {
            return StatusCode::UNPROCESSABLE_ENTITY.into_response();
        }
        if id == 0 {
            return Redirect::to(back).into_response();
        }
        return Redirect::to(&format!("/admin/comments/edit/{id}?m=empty")).into_response();
    }
    let get = |k: &str| f.get(k).cloned().unwrap_or_default();
    // Keep the existing status — quick edit never sets it.
    let (status, post_id) = match crud::comments::get(&sc.db, id) {
        Ok(Some(r)) => (r.status, r.post_id.unwrap_or(0)),
        _ => return Redirect::to(back).into_response(),
    };
    let _ = crud::comments::update_fields(&sc.db, id, &get("author"), &get("email"), &get("url"), &body, &status, None);
    admin::regen_posts(sc, &[post_id]);
    if htmx {
        // `cls` rides the qe form (the origin row's class attribute) so the
        // replacement keeps its alternating/thread/depth parity tokens.
        let cols = Cols::load(&sc.db, s.user_id);
        let cls = clean_cls(&get("cls"));
        return match crud::comments::get(&sc.db, id) {
            Ok(Some(r)) => Html(row_html(&sc, &r, &cols, &cls)).into_response(),
            _ => StatusCode::NOT_FOUND.into_response(),
        };
    }
    redirect_back(back, "edited", 1)
}

/// GET /admin/comments/reply/{id} — WP's inline `.reply-row`: htmx fetches a
/// `<tr>` fragment inserted `afterend` the comment row; a plain navigation
/// (no JS) still gets the standalone reply page.
pub(crate) async fn reply_page(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    let Ok(Some(r)) = crud::comments::get(&sc.db, id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if is_htmx(&headers) {
        // Fields associate with the out-of-table #cmt-reply form (a <form>
        // inside <tbody> would be dropped by the HTML parser). `cls` is the
        // parent row's parity tokens for the reply row that replaces this.
        let cls = esc_attr(&clean_cls(q.get("cls").map(|s| s.as_str()).unwrap_or("")));
        let frag = format!(
            r##"<tr class="reply-row" style="display:table-row"><td colspan="5" class="colspanchange">
<input form="cmt-reply" type="hidden" name="parent" value="{id}">
<input form="cmt-reply" type="hidden" name="cls" value="{cls}">
<fieldset><legend class="inline-edit-legend">Reply to Comment</legend>
<p style="margin:0 0 6px"><strong>{author}</strong> wrote:</p>
<div style="background:#fff;border:1px solid #dcdcde;padding:10px;margin-bottom:10px;max-width:700px;white-space:pre-wrap">{body}</div>
<label for="reply-body" class="screen-reader-text">Reply</label>
<textarea form="cmt-reply" name="body" id="reply-body" class="large-text" rows="5" autofocus></textarea>
<p class="submit inline-edit-save" style="margin-top:10px">
<button form="cmt-reply" type="submit" class="button button-primary" hx-post="/admin/comments/reply" hx-include="#cmt-reply" hx-target="closest tr" hx-swap="outerHTML">Submit Reply</button>
<button type="button" class="button" onclick="this.closest('tr').remove()">Cancel</button>
</p>
</fieldset>
</td></tr>"##,
            id = r.id,
            cls = cls,
            author = esc(&r.author),
            body = esc(&r.body),
        );
        return Html(frag).into_response();
    }
    let inner = format!(
        r##"{chrome}<div class="wrap"><h1 class="wp-heading-inline">Reply to Comment</h1><hr class="wp-header-end">
<div class="comment-reply-preview" style="background:#fff;border:1px solid #dcdcde;padding:14px;margin:14px 0;max-width:700px">
<strong>{author}</strong><br><span style="white-space:pre-wrap">{body}</span></div>
<form method="post" action="/admin/comments/reply">
<input type="hidden" name="csrf" value="{csrf}">
<input type="hidden" name="parent" value="{id}">
<p><label for="reply-body"><strong>Reply</strong></label><br>
<textarea name="body" id="reply-body" class="large-text" rows="8" autofocus></textarea></p>
<p class="submit"><input type="submit" class="button button-primary" value="Submit Reply">
<a class="button" href="/admin/comments">Cancel</a></p>
</form></div>"##,
        chrome = chrome("comments", &s),
        csrf = s.csrf,
        id = r.id,
        author = esc(&r.author),
        body = esc(&r.body),
    );
    html_admin(&s, "Reply to Comment", &inner, false, "comment-php").into_response()
}

/// POST /admin/comments/reply — creates an approved child; author = session
/// user's display_name (email fallback), never form input. htmx requests get
/// the new child `<tr>` back — it replaces the `.reply-row` form
/// (`hx-swap="outerHTML"`); plain POSTs redirect like before.
pub(crate) async fn reply_save(
    State(app): State<App>,
    headers: HeaderMap,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if f.get("csrf").map(|x| x.as_str()) != Some(s.csrf.as_str()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let htmx = is_htmx(&headers);
    let parent = f.get("parent").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
    let body = f.get("body").cloned().unwrap_or_default();
    if parent == 0 || body.trim().is_empty() {
        if htmx {
            return StatusCode::UNPROCESSABLE_ENTITY.into_response();
        }
        return Redirect::to("/admin/comments?m=empty").into_response();
    }
    let author = crud::comments::user_display_name(&sc.db, s.user_id, &s.email);
    let new_id = crud::comments::insert_reply(&sc.db, parent, &author, &s.email, &body).unwrap_or(0);
    // A new approved reply changes the parent post's page.
    let reply_post = crud::comments::get(&sc.db, new_id)
        .ok()
        .flatten()
        .and_then(|r| r.post_id)
        .unwrap_or(0);
    admin::regen_posts(sc, &[reply_post]);
    if htmx {
        let cols = Cols::load(&sc.db, s.user_id);
        // The reply is a child of the row it was opened under → depth+1.
        let cls = clean_cls(f.get("cls").map(|s| s.as_str()).unwrap_or(""));
        let cls = if cls.contains("depth-") { cls.replace("depth-1", "depth-2") } else { format!("{cls} depth-2") };
        return match crud::comments::get(&sc.db, new_id) {
            Ok(Some(r)) => Html(row_html(&sc, &r, &cols, &cls)).into_response(),
            _ => Redirect::to("/admin/comments?m=replied").into_response(),
        };
    }
    Redirect::to("/admin/comments?m=replied").into_response()
}

/// POST /admin/comments/screen — Screen Options "Apply": per-user column
/// visibility as JSON in `settings.comment_screen_{user_id}`.
/// Route pending: `post("/admin/comments/screen")` registered by the router owner.
#[allow(dead_code)]
pub(crate) async fn screen_options(
    State(app): State<App>,
    headers: HeaderMap,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if f.get("csrf").map(|x| x.as_str()) != Some(s.csrf.as_str()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let back = list::back_or(&f, "/admin/comments");
    let cols = Cols {
        author: f.contains_key("author-hide"),
        comment: f.contains_key("comment-hide"),
        response: f.contains_key("response-hide"),
        date: f.contains_key("date-hide"),
    };
    if let Ok(json) = serde_json::to_string(&cols) {
        let _ = crud::comments::setting_set(&sc.db, &format!("comment_screen_{}", s.user_id), &json);
    }
    Redirect::to(back).into_response()
}
