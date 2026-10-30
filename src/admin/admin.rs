#[allow(unused_imports)]
use crate::sql;
use super::*;
pub(crate) async fn htmx(State(app): State<App>, headers: HeaderMap) -> Response {
    let sc = site_for(&app, &headers);
    let p = sc.templates.join("admin/vendor/htmx.min.js");
    // Vendored asset: read + memoize once — same bytes for the process
    // lifetime; `immutable` so the browser never revalidates.
    let b = {
        static CACHED: std::sync::LazyLock<Mutex<Option<Arc<Vec<u8>>>>> =
            std::sync::LazyLock::new(|| Mutex::new(None));
        let mut g = CACHED.lock();
        g.get_or_insert_with(|| Arc::new(std::fs::read(&p).unwrap_or_default())).clone()
    };
    if b.is_empty() {
        return StatusCode::NOT_FOUND.into_response();
    }
    let body = axum::body::Body::from((*b).clone());
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8".to_string()), (header::CACHE_CONTROL, "public, max-age=31536000, immutable".to_string())],
        body,
    )
        .into_response()
}

#[derive(Deserialize)]
pub(crate) struct CsrfForm {
    csrf: String,
}

pub(crate) async fn api_generate(
    State(app): State<App>,
    headers: HeaderMap,
    Form(form): Form<CsrfForm>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
        if let Err(r) = check_csrf(&s, &form.csrf) {
        return r;
    }
    if sc.generating.swap(true, Ordering::SeqCst) {
        return (StatusCode::CONFLICT, "already running").into_response();
    }
    let db = sc.db.clone();
    let templates = sc.templates.clone();
    let out = sc.out.clone();
    let flag = sc.generating.clone();
    let tour_base = crate::admin::tour_base(&sc).to_string();
    tokio::task::spawn_blocking(move || {
        // "Rebuild site files" under the render-first model: NOT a page
        // bake — just the scheduled duties + utility files + cache bust.
        // 1. WP `_wp_scheduled_delete` equivalent: trash > 30 days dies.
        crate::sitefx::purge_duties(&db, &out, &tour_base);
        // 2. Emit sitemap/feeds/robots/404/search/redirects.map.
        let base = crate::db::setting_value(&db, "site_url", "/");
        if let Err(e) = crate::sitefx::emit_site_files(&db, &templates, &out, &base, &tour_base) {
            eprintln!("emit_site_files: {e:#}");
        }
        // 3. Clear the page cache — it re-warms itself from traffic.
        crate::admin::admin::bust_cache_dir(&out);
        // Keep `widgets_dirty` honest: a full bust means nothing pending.
        let _ = crate::db::setting_put(&db, "widgets_dirty", "0");
        flag.store(false, Ordering::SeqCst);
    });
    Redirect::to("/admin").into_response()
}

/// Full-site regen used by settings screens + non-form callers: same
/// body as `api_generate` (flag + spawn_blocking duties + emit + bust).
/// Returns `false` when a regen is already running.
pub(crate) fn regen_site(sc: &SiteCtx) -> bool {
    if sc.generating.swap(true, Ordering::SeqCst) {
        return false;
    }
    let db = sc.db.clone();
    let templates = sc.templates.clone();
    let out = sc.out.clone();
    let flag = sc.generating.clone();
    let tour_base = crate::admin::tour_base(sc).to_string();
    tokio::task::spawn_blocking(move || {
        crate::sitefx::purge_duties(&db, &out, &tour_base);
        let base = crate::db::setting_value(&db, "site_url", "/");
        if let Err(e) = crate::sitefx::emit_site_files(&db, &templates, &out, &base, &tour_base) {
            eprintln!("emit_site_files: {e:#}");
        }
        crate::admin::admin::bust_cache_dir(&out);
        let _ = crate::db::setting_put(&db, "widgets_dirty", "0");
        flag.store(false, Ordering::SeqCst);
    });
    true
}

pub(crate) async fn generate_status(State(app): State<App>, headers: HeaderMap) -> Response {
    let Ok((_s, sc)) = need_auth(&app, &headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let running = sc.generating.load(Ordering::SeqCst);
    (
        [(header::CONTENT_TYPE, "application/json")],
        format!(r#"{{"running":{running}}}"#),
    )
        .into_response()
}

pub(crate) async fn api_posts(State(app): State<App>, headers: HeaderMap) -> Response {
    let Ok((_s, sc)) = need_auth(&app, &headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let conn = match crate::db::conn(&sc.db) {
        Ok(c) => c,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let n: i64 = conn
        .query_row(
            sql!("admin__select_count_from"),
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    format!(r#"{{"count":{n}}}"#).into_response()
}

/// Media library — WP `/wp-admin/upload.php` list + grid modes.
/// Query params: mode=list|grid, type=<group> (mime= alias), m=YYYY-MM|YYYY/MM,
/// s=search, sort=date|date_asc|title, paged=N.
/// Persists: `settings.media_view_{user_id}` (list|grid) and
/// `settings.media_screen_{user_id}` ({"hide":[cols],"per_page":n}).
pub(crate) async fn media_page(State(app): State<App>, headers: HeaderMap, Query(q): Query<HashMap<String, String>>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    // View toggle: explicit ?mode= wins and is persisted; otherwise the last
    // stored choice applies so the toggle survives a reload.
    let view_key = format!("media_view_{}", s.user_id);
    let mode = match q.get("mode").map(|v| v.as_str()) {
        Some(m @ ("list" | "grid")) => {
            let _ = crud::comments::setting_set(&sc.db, &view_key, m);
            m
        }
        _ => match crud::comments::setting_get(&sc.db, &view_key).as_deref() {
            Some("grid") => "grid",
            _ => "list",
        },
    };
    // Spec names the mime filter `type`; `mime` stays as the picker alias.
    let mime = q.get("type").or_else(|| q.get("mime")).map(|v| v.as_str()).unwrap_or("");
    // Accept both stored `created` format (YYYY/MM) and the spec's YYYY-MM.
    let month_raw = q.get("m").map(|v| v.as_str()).unwrap_or("");
    let month;
    if month_raw.len() == 7 && month_raw.as_bytes()[4] == b'-' {
        month = month_raw.replacen('-', "/", 1);
    } else {
        month = month_raw.to_string();
    }
    let month = month.as_str();
    let search = q.get("s").map(|v| v.as_str()).unwrap_or("");
    let sort = q.get("sort").map(|v| v.as_str()).unwrap_or("date");
    // Clamp: u32::MAX pages is more than any library; avoids offset overflow.
    let page: i64 = q.get("paged").and_then(|v| v.parse().ok()).unwrap_or(1).clamp(1, 4_000_000_000);

    // Query-string builder preserving active filters.
    let mqs = |over: &[(&str, &str)]| {
        let mut parts: Vec<(String, String)> = vec![
            ("mode".into(), mode.into()),
            ("type".into(), mime.into()),
            ("m".into(), month.into()),
            ("s".into(), search.into()),
            ("sort".into(), sort.into()),
            ("paged".into(), page.to_string()),
        ];
        for (k, v) in over {
            parts.iter_mut().for_each(|p| {
                if p.0 == *k {
                    p.1 = v.to_string();
                }
            });
        }
        parts
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(k, v)| format!("{}={}", k, urlenc(v)))
            .collect::<Vec<_>>()
            .join("&")
    };

    // Screen Options (GET form like edit-tags.php): `screen_options=1` marks a
    // submit so absent `hide_*` checkboxes mean "hidden". Columns default
    // visible; per_page clamps to a sane WP range.
    const MEDIA_COLS: [&str; 5] = ["author", "parent", "date", "size", "dimensions"];
    let screen_key = format!("media_screen_{}", s.user_id);
    if q.get("screen_options").is_some() {
        let hide: Vec<String> = MEDIA_COLS
            .iter()
            .filter(|c| !q.contains_key(&format!("hide_{c}")))
            .map(|c| c.to_string())
            .collect();
        let per = q.get("per_page").and_then(|v| v.parse::<i64>().ok()).unwrap_or(20).clamp(1, 500);
        let json = serde_json::json!({"hide": hide, "per_page": per}).to_string();
        let _ = crud::comments::setting_set(&sc.db, &screen_key, &json);
        return Redirect::to(&format!("/admin/media?{}", mqs(&[]))).into_response();
    }
    let (hide_cols, per_page): (Vec<String>, i64) = crud::comments::setting_get(&sc.db, &screen_key)
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .map(|v| {
            let hide = v.get("hide").and_then(|h| h.as_array()).map(|a| {
                a.iter().filter_map(|x| x.as_str().map(String::from)).collect::<Vec<_>>()
            }).unwrap_or_default();
            let per = v.get("per_page").and_then(|p| p.as_i64()).unwrap_or(20).clamp(1, 500);
            (hide, per)
        })
        .unwrap_or((Vec::new(), 20));
    let col_cls = |name: &str| if hide_cols.iter().any(|h| h == name) { " hidden" } else { "" };

    let (rows, total) = match crud::media_list(&sc.db, mime, month, search, sort, (page - 1) * per_page, per_page) {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")).into_response(),
    };
    let pages = ((total + per_page - 1) / per_page).max(1);

    let months_opts = {
        let mut o = String::from(r#"<option value="">All dates</option>"#);
        for m in crud::media_months(&sc.db).unwrap_or_default() {
            let sel = if m == month { " selected" } else { "" };
            o.push_str(&format!(r#"<option value="{}"{}>{}</option>"#, esc(&m), sel, esc(&m)));
        }
        o
    };

    let mime_opts = {
        let mut opts: Vec<(&str, &str)> = vec![
            ("", "All media items"), ("image", "Images"),
            ("application/pdf", "PDF"), ("other", "Other"),
        ];
        // Unknown/legacy filter value still applied → surface it in the
        // select so the UI doesn't claim "All media items" while filtering.
        if !mime.is_empty() && !opts.iter().any(|(v, _)| *v == mime) {
            opts.push((mime, mime));
        }
        opts.iter()
            .map(|(v, l)| format!(r#"<option value="{}"{}>{}</option>"#, esc(v), if *v == mime { " selected" } else { "" }, esc(l)))
            .collect::<String>()
    };

    let at_first = if page <= 1 { " disabled" } else { "" };
    let at_last = if page >= pages { " disabled" } else { "" };
    let pager = format!(
        r#"<div class="tablenav-pages"><span class="displaying-num">{total} items</span>
<span class="pagination-links">
<a class="first-page button{at_first}" href="/admin/media?{q1}"><span>«</span></a>
<a class="prev-page button{at_first}" href="/admin/media?{qp}"><span>‹</span></a>
<span class="paging-input"><span class="tablenav-paging-text">{page} of <span class="total-pages">{pages}</span></span></span>
<a class="next-page button{at_last}" href="/admin/media?{qn}"><span>›</span></a>
<a class="last-page button{at_last}" href="/admin/media?{ql}"><span>»</span></a>
</span></div>"#,
        q1 = mqs(&[("paged", "1")]),
        qp = mqs(&[("paged", &(page - 1).max(1).to_string())]),
        qn = mqs(&[("paged", &(page + 1).min(pages).to_string())]),
        ql = mqs(&[("paged", &pages.to_string())]),
    );

    let view_switch = format!(
        r##"<div class="view-switch"><a href="/admin/media?{ql}" class="view-list{cl}"><span class="screen-reader-text">List view</span></a>
<a href="/admin/media?{qg}" class="view-grid{cg}"><span class="screen-reader-text">Grid view</span></a></div>"##,
        ql = mqs(&[("mode", "list"), ("paged", "1")]),
        qg = mqs(&[("mode", "grid")]),
        cl = if mode == "list" { " current" } else { "" },
        cg = if mode == "grid" { " current" } else { "" },
    );
    // Redirect target for POST actions (bulk delete, single-row delete).
    let back = esc(&format!("/admin/media?{}", mqs(&[])));

    let table_head = format!(
        r#"<tr><td class="manage-column column-cb check-column"></td>
<th scope="col" class="manage-column column-title column-primary">File</th>
<th scope="col" class="manage-column column-author{c_author}">Author</th>
<th scope="col" class="manage-column column-parent{c_parent}">Uploaded to</th>
<th scope="col" class="manage-column column-date{c_date}">Date</th>
<th scope="col" class="manage-column column-size{c_size}">File Size</th>
<th scope="col" class="manage-column column-dimensions{c_dimensions}">Dimensions</th></tr>"#,
        c_author = col_cls("author"),
        c_parent = col_cls("parent"),
        c_date = col_cls("date"),
        c_size = col_cls("size"),
        c_dimensions = col_cls("dimensions"),
    );

    let mut body = String::new();
    let mut grid = String::new();
    let attach_map = crud::media_attached_map(&sc.db).unwrap_or_default();
    for m in &rows {
        let url = format!("/wp-content/uploads/{}", m.path);
        let file_name = m.path.rsplit('/').next().unwrap_or(&m.path);
        let display_title = if m.title.is_empty() { file_name } else { &m.title };
        let is_image = m.mime.starts_with("image/");
        let thumb = if is_image {
            format!(r#"<img src="{}" width="60" height="60" style="width:60px;height:60px;object-fit:cover" alt="">"#, esc(&url))
        } else {
            format!(r#"<span class="dashicons {}"></span>"#, mime_icon(&m.mime))
        };
        let size = media_size(sc, m);
        let size_d = size.map(human_size).unwrap_or_else(|| "—".into());
        let dims_d = match (m.w, m.h) {
            (Some(w), Some(h)) => format!("{w}×{h}"),
            _ => "—".into(),
        };
        let date_d = if m.created.is_empty() {
            m.path.get(..7).unwrap_or("—").replace('/', " ").to_string()
        } else {
            m.created.clone()
        };
        let attached = attach_map.get(&m.id).cloned().unwrap_or_default();
        let uploaded_to = if attached.is_empty() {
            "<span class=\"uploaded-to-empty\">(Unattached)</span>".to_string()
        } else {
            attached
                .iter()
                .map(|(pid, t)| format!(
                    r#"<a href="/admin/edit/{pid}">{t}</a>
<form method="post" action="/admin/media/detach" style="display:inline"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="media_id" value="{mid}"><input type="hidden" name="post_id" value="{pid}"><button class="button-link">Detach</button></form>"#,
                    t = esc(t),
                    csrf = s.csrf,
                    mid = m.id
                ))
                .collect::<Vec<_>>()
                .join("<br>")
        };
        // Row Edit opens the same attachment-details modal as grid mode
        // (`att_open` is exposed by grid_modal's script); Delete Permanently
        // rides the existing media-bulk endpoint with a single m-{id} flag.
        let actions = format!(
            r##"<div class="row-actions"><span class="edit"><a href="/admin/media/edit/{id}" onclick="att_open({id});return false">Edit</a> | </span><span class="delete"><form method="post" action="/admin/media/bulk" style="display:inline" onsubmit="return confirm('Delete permanently?')"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="action" value="delete"><input type="hidden" name="back" value="{back}"><input type="hidden" name="m-{id}" value="1"><button class="button-link submitdelete">Delete Permanently</button></form> | </span><span class="view"><a href="{url}" target="_blank">View</a> | </span><span class="copy"><a href="#" onclick="navigator.clipboard.writeText('{abs}');this.textContent='Copied!';return false">Copy URL</a> | </span><span class="download"><a href="/admin/media/download/{id}">Download file</a></span></div>"##,
            id = m.id,
            csrf = s.csrf,
            url = esc(&url),
            back = back,
            abs = esc_js(&format!("{}{}", site_url(&sc).trim_end_matches('/'), url)),
        );
        body.push_str(&format!(
            r#"<tr id="media-{id}"><th scope="row" class="check-column"><input form="media-bulk" type="checkbox" name="m-{id}"></th>
<td class="has-row-actions column-primary">
<a href="/admin/media/edit/{id}" class="media-icon" style="float:left;margin:0 9px 0 0">{thumb}</a>
<p class="title" style="word-break:break-all"><a href="/admin/media/edit/{id}"><strong>{title}</strong></a></p>
{actions}</td>
<td class="author column-author{c_author}">{author}</td>
<td class="parent column-parent{c_parent}">{uploaded_to}</td>
<td class="date column-date{c_date}">{date}</td>
<td class="size column-size{c_size}">{size}</td>
<td class="dimensions column-dimensions{c_dimensions}">{dims}</td></tr>"#,
            id = m.id,
            thumb = thumb,
            title = esc(display_title),
            actions = actions,
            author = esc(if m.owner.is_empty() { &s.email } else { &m.owner }),
            uploaded_to = uploaded_to,
            date = esc(&date_d),
            size = size_d,
            dims = dims_d,
            c_author = col_cls("author"),
            c_parent = col_cls("parent"),
            c_date = col_cls("date"),
            c_size = col_cls("size"),
            c_dimensions = col_cls("dimensions"),
        ));
        grid.push_str(&format!(
            r#"<li class="attachment" data-id="{id}" style="display:inline-block;width:120px;height:120px;margin:6px;vertical-align:top;text-align:center;cursor:pointer;position:relative" title="{t}"><span class="media-icon" style="display:block;width:120px;height:120px;overflow:hidden;border:1px solid #dcdcde;background:#f0f0f1">{thumb}</span><input form="media-bulk" type="checkbox" class="att-check" name="m-{id}" style="display:none;position:absolute;top:2px;right:2px;z-index:2;width:18px;height:18px"></li>"#,
            id = m.id,
            t = esc(display_title),
            thumb = if is_image {
                format!(r#"<img src="{}" style="width:100%;height:100%;object-fit:cover" alt="">"#, esc(&url))
            } else {
                format!(r#"<span class="dashicons {ic}" style="font-size:48px;line-height:120px"></span>"#, ic = mime_icon(&m.mime))
            },
        ));
    }
    if body.is_empty() {
        body.push_str(r#"<tr class="no-items"><td class="colspanchange" colspan="7">No media files found.</td></tr>"#);
    }

    let mut notice = String::new();
    if let Some(d) = q.get("deleted").and_then(|v| v.parse::<i64>().ok()).filter(|n| *n > 0) {
        notice = format!(r#"<div class="notice notice-success is-dismissible"><p>{d} media attachment{} deleted permanently.</p></div>"#, if d == 1 { "" } else { "s" });
    } else if let Some(done) = q.get("regen") {
        let skipped = q.get("skipped").map(|v| v.as_str()).unwrap_or("0");
        notice = format!(r#"<div class="notice notice-success is-dismissible"><p>Regenerated image sizes for {done} attachment{} ({skipped} skipped — remote or non-image).</p></div>"#, if done == "1" { "" } else { "s" });
    } else if q.get("updated").is_some() {
        notice = r#"<div class="notice notice-success is-dismissible"><p>Media attachment updated.</p></div>"#.to_string();
    }

    // Attachment-details modal markup+JS — emitted in both modes so the list
    // row Edit links can open it via `att_open(id)`.
    let modal = grid_modal(&rows, &s.csrf);
    let list_markup = format!(
        r##"<form method="get" action="/admin/media">
<input type="hidden" name="mode" value="list">
<div class="tablenav top">
<div class="alignleft actions bulkactions">
<label for="bulk-action-selector-top" class="screen-reader-text">Select bulk action</label>
<select form="media-bulk" name="action" id="bulk-action-selector-top">
<option value="-1">Bulk actions</option>
<option value="delete">Delete permanently</option>
</select>
<input form="media-bulk" type="submit" id="doaction" class="button action" value="Apply">
</div>
<div class="alignleft actions">
<select name="type" id="filter-by-mime">{mime_opts}</select>
<select name="m" id="filter-by-date">{months_opts}</select>
<input type="submit" name="filter_action" id="post-query-submit" class="button" value="Filter">
<button form="media-regen" type="submit" class="button" onclick="return confirm('Rebuild all thumbnail/medium/large files and metadata for the library?')">Regenerate Sizes</button>
</div>
{view_switch}
<p class="search-box"><label class="screen-reader-text" for="media-search-input">Search Media:</label>
<input type="search" id="media-search-input" name="s" value="{search_val}">
<input type="submit" id="search-submit" class="button" value="Search Media"></p>
{pager}
<br class="clear">
</div>
</form>
<form method="post" action="/admin/media/bulk" id="media-bulk"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="back" value="{back}"></form>
<form method="post" action="/admin/media/regenerate" id="media-regen"><input type="hidden" name="csrf" value="{csrf}"></form>
<table class="wp-list-table widefat fixed striped table-view-list media">
<thead>{head}</thead><tbody id="the-list">{body}</tbody><tfoot>{head}</tfoot></table>
{modal}"##,
        mime_opts = mime_opts,
        months_opts = months_opts,
        pager = pager,
        search_val = esc(search),
        view_switch = view_switch,
        csrf = s.csrf,
        back = back,
        head = table_head,
        body = body,
        modal = modal,
    );

    let content = if mode == "grid" {
        format!(
            r##"<form method="get" action="/admin/media">
<input type="hidden" name="mode" value="grid">
<div class="tablenav top">
{view_switch}
<button type="button" class="button media-button select-mode-toggle-button" id="att-bulk-btn">Bulk Select</button>
<div class="alignleft actions">
<select name="type" id="filter-by-mime">{mime_opts}</select>
<select name="m" id="filter-by-date">{months_opts}</select>
<input type="submit" name="filter_action" id="post-query-submit" class="button" value="Filter">
</div>
<p class="search-box"><label class="screen-reader-text" for="media-search-input">Search Media:</label>
<input type="search" id="media-search-input" name="s" value="{search_val}">
<input type="submit" id="search-submit" class="button" value="Search Media"></p>
{pager}
<br class="clear">
</div>
</form>
<form method="post" action="/admin/media/bulk" id="media-bulk"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="action" value="delete"><input type="hidden" name="back" value="{back}"></form>
<ul class="attachments" id="att-grid" style="list-style:none;margin:0;padding:10px 0">{grid}</ul>
<div id="att-bulk-bar" style="display:none;position:fixed;left:160px;right:0;bottom:0;background:#1d2327;color:#fff;padding:10px 16px;z-index:9998;box-shadow:0 -2px 8px rgba(0,0,0,.25)">
<span id="att-bulk-count" style="margin-right:12px">0 selected</span>
<button type="submit" form="media-bulk" class="button button-primary" id="att-bulk-del" disabled onclick="return confirm('Delete the selected items permanently?')">Delete Selected</button>
<button type="button" class="button" id="att-bulk-cancel">Cancel</button>
</div>
{modal}"##,
            mime_opts = mime_opts,
            months_opts = months_opts,
            pager = pager,
            search_val = esc(search),
            view_switch = view_switch,
            csrf = s.csrf,
            back = back,
            grid = grid,
            modal = modal,
        )
    } else {
        list_markup
    };

    let inner = format!(
        r#"{chrome}{screen_meta}<style>
#screen-meta-links{{float:right;margin:0 20px 0 0}}
#screen-meta-links .show-settings{{border:1px solid #c3c4c7;border-top:0;background:#f6f7f7;padding:3px 12px;border-radius:0 0 4px 4px}}
#screen-options-wrap{{background:#fff;border:1px solid #c3c4c7;padding:15px 20px;box-shadow:0 2px 5px rgba(0,0,0,.15)}}
#screen-options-wrap.hidden,.wp-list-table .hidden{{display:none}}
.attachments .attachment.att-sel{{outline:3px solid #2271b1;outline-offset:-3px}}
</style><div class="wrap"><h1 class="wp-heading-inline">Media Library</h1>
<a href="/admin/media/new" class="page-title-action">Add Media File</a>
<hr class="wp-header-end">{notice}{content}</div>
<script>
(function(){{
var btn=document.getElementById('show-settings-link');
if(btn)btn.addEventListener('click',function(){{
  var w=document.getElementById('screen-options-wrap');
  var hid=w.classList.toggle('hidden');
  btn.setAttribute('aria-expanded',hid?'false':'true');
}});
// WP hide-column-tog: toggle .hidden on every .column-* cell live;
// persistence happens via the screen-options GET submit (screen_options=1).
document.addEventListener('change',function(e){{
  var t=e.target;
  if(!t||!t.classList||!t.classList.contains('hide-column-tog'))return;
  var cells=document.querySelectorAll('.column-'+t.value);
  for(var i=0;i<cells.length;i++)cells[i].classList.toggle('hidden',!t.checked);
}});
// Grid "Bulk Select" mode: thumbs carry a hidden .att-check; in bulk mode a
// click toggles the checkbox instead of opening the attachment modal, and a
// fixed bottom bar offers Delete Selected / Cancel.
var bulkBtn=document.getElementById('att-bulk-btn');
if(bulkBtn){{
  var bar=document.getElementById('att-bulk-bar');
  var del=document.getElementById('att-bulk-del');
  var cnt=document.getElementById('att-bulk-count');
  function checks(){{return document.querySelectorAll('#att-grid .att-check');}}
  function refresh(){{
    var n=0;checks().forEach(function(c){{if(c.checked)n++;}});
    cnt.textContent=n+' selected';del.disabled=n===0;
  }}
  function setMode(on){{
    window._attBulk=on;
    bar.style.display=on?'block':'none';
    checks().forEach(function(c){{c.style.display=on?'block':'none';if(!on)c.checked=false;c.closest('.attachment').classList.remove('att-sel');}});
    bulkBtn.classList.toggle('button-primary',on);
    if(!on)refresh();
  }}
  bulkBtn.addEventListener('click',function(){{setMode(!window._attBulk);}});
  document.getElementById('att-bulk-cancel').addEventListener('click',function(){{setMode(false);}});
  document.getElementById('att-grid').addEventListener('click',function(e){{
    if(!window._attBulk)return;
    var li=e.target.closest('.attachment');if(!li)return;
    e.preventDefault();e.stopPropagation();
    var c=li.querySelector('.att-check');
    if(e.target!==c)c.checked=!c.checked;
    li.classList.toggle('att-sel',c.checked);
    refresh();
  }},true);
}}
}})();
</script>"#,
        chrome = chrome("media", &s),
        screen_meta = media_screen_meta(&hide_cols, per_page, &s.csrf, mime, month, search, sort, mode),
        notice = notice,
        content = content,
    );
    html_page(&s, "Media", &inner, true).into_response()
}

/// WP `#screen-meta` + `#screen-meta-links` for the media screen: "Screen
/// Options" tab toggling a GET form of `hide_*` checkboxes + per_page. A GET
/// submit re-enters `media_page` with `screen_options=1`, which persists to
/// `settings.media_screen_{user_id}` — no dedicated route needed.
fn media_screen_meta(hide: &[String], per_page: i64, csrf: &str, mime: &str, month: &str, search: &str, sort: &str, mode: &str) -> String {
    let cb = |name: &str, label: &str| {
        let on = !hide.iter().any(|h| h == name);
        format!(
            "<label for=\"{name}-hide\"><input class=\"hide-column-tog\" name=\"hide_{name}\" type=\"checkbox\" id=\"{name}-hide\" value=\"{name}\"{}>{label}</label>",
            if on { " checked" } else { "" }
        )
    };
    format!(
        r#"<div id="screen-meta" class="metabox-prefs">
<div id="screen-options-wrap" class="hidden" tabindex="-1" aria-label="Screen Options Tab">
<form id="adv-settings" method="get" action="/admin/media">
<input type="hidden" name="screen_options" value="1">
<input type="hidden" name="csrf" value="{csrf}">
<input type="hidden" name="type" value="{mime}">
<input type="hidden" name="m" value="{month}">
<input type="hidden" name="s" value="{search}">
<input type="hidden" name="sort" value="{sort}">
<input type="hidden" name="mode" value="{mode}">
<fieldset class="metabox-prefs view-options">
<legend>Columns</legend>
{cb_author}
{cb_parent}
{cb_date}
{cb_size}
{cb_dimensions}
</fieldset>
<fieldset class="screen-options"><legend>Pagination</legend>
<label for="media_per_page">Number of items per page:</label>
<input type="number" step="1" min="1" max="500" class="screen-per-page" name="per_page" id="media_per_page" value="{per_page}">
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
        mime = esc(mime),
        month = esc(month),
        search = esc(search),
        sort = esc(sort),
        mode = esc(mode),
        cb_author = cb("author", "Author"),
        cb_parent = cb("parent", "Uploaded to"),
        cb_date = cb("date", "Date"),
        cb_size = cb("size", "File Size"),
        cb_dimensions = cb("dimensions", "Dimensions"),
        per_page = per_page,
    )
}

/// WP `/wp-admin/media-new.php` — multi-file drop zone page.
pub(crate) async fn media_new_page(State(app): State<App>, headers: HeaderMap) -> Response {
    let Ok((s, _sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    let inner = format!(
        r##"{chrome}<div class="wrap"><h1 class="wp-heading-inline">Upload Media</h1>
<hr class="wp-header-end">
<div id="media-upload-error"></div>
<div class="media-upload-form">
<div class="drag-drop" id="drag-drop-area">
<div class="drag-drop-inside">
<p class="drag-drop-info">Drop files to upload</p>
<p>or</p>
<p class="upload-flash-bypass"><button type="button" class="button button-hero" id="select-files" style="position:relative;z-index:1">Select Files</button></p>
</div>
</div>
<form method="post" action="/admin/media/upload" enctype="multipart/form-data" id="file-form" style="display:none">
<input type="hidden" name="csrf" value="{csrf}">
<input type="file" name="file" id="file" multiple>
</form>
<form method="post" action="/admin/media/upload" enctype="multipart/form-data" id="html-uploader">
<input type="hidden" name="csrf" value="{csrf}">
<p>You are using the multi-file uploader. Problems? <label style="text-decoration:underline;cursor:pointer">Try the browser uploader <input type="file" name="file" style="width:0;height:0;opacity:0" onchange="this.form.submit()"></label> instead.</p>
</form>
<div id="media-items"></div>
<p class="max-upload-size">Maximum upload file size: {upload_limit}.</p>
</div></div>
<style>.drag-drop{{border:4px dashed #c3c4c7;min-height:300px;display:flex;align-items:center;justify-content:center;text-align:center;margin:20px 0}}
.drag-drop.dragover{{border-color:#2271b1;background:#f0f6fb}}
.drag-drop-info{{font-size:22px;color:#3c434a}}
.media-item{{border:1px solid #dcdcde;padding:10px;margin:10px 0;background:#fff}}
.media-item .bar{{height:6px;background:#dcdcde;margin-top:6px}}
.media-item .bar div{{height:6px;background:#2271b1;width:0%}}</style>
<script>
(function(){{
var zone=document.getElementById('drag-drop-area');
var file=document.getElementById('file');
var items=document.getElementById('media-items');
document.getElementById('select-files').addEventListener('click',function(){{file.click();}});
file.addEventListener('change',function(){{upload(file.files);file.value='';}});
zone.addEventListener('dragover',function(e){{e.preventDefault();zone.classList.add('dragover');}});
zone.addEventListener('dragleave',function(){{zone.classList.remove('dragover');}});
zone.addEventListener('drop',function(e){{e.preventDefault();zone.classList.remove('dragover');upload(e.dataTransfer.files);}});
function upload(files){{
  var queue=[].slice.call(files), done=0;
  function next(){{
    if(!queue.length){{location.href='/admin/media';return;}}
    var f=queue.shift();
    var div=document.createElement('div');div.className='media-item';
    div.innerHTML='<strong>'+f.name+'</strong><span class="st"></span><div class="bar"><div></div></div>';
    items.appendChild(div);
    var fd=new FormData();
    fd.append('csrf','{csrf}');
    fd.append('file',f);
    var xhr=new XMLHttpRequest();
    xhr.open('POST','/admin/media/upload');
    xhr.upload.addEventListener('progress',function(e){{if(e.lengthComputable)div.querySelector('.bar div').style.width=(100*e.loaded/e.total)+'%';}});
    xhr.onload=function(){{
      if(xhr.status<300){{div.querySelector('.st').textContent=' — uploaded';}}
      else{{div.querySelector('.st').textContent=' — error: '+xhr.responseText;}}
      if(++done>=0)next();
    }};
    xhr.send(fd);
  }}
  next();
}}
}})();
</script>"##,
        chrome = chrome("media", &s),
        csrf = s.csrf,
        upload_limit = crate::db::UPLOAD_LIMIT_DISPLAY,
    );
    html_page(&s, "Upload Media", &inner, true).into_response()
}

/// WP `post.php?post=<id>&action=edit` for attachments.
pub(crate) async fn media_edit_page(State(app): State<App>, headers: HeaderMap, Path(id): Path<i64>, Query(q): Query<HashMap<String, String>>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    let Some(m) = crud::media_get(&sc.db, id).ok().flatten() else {
        return (StatusCode::NOT_FOUND, "media not found").into_response();
    };
    let url = format!("/wp-content/uploads/{}", m.path);
    let abs = format!("{}{}", site_url(&sc).trim_end_matches('/'), url);
    let file_name = m.path.rsplit('/').next().unwrap_or(&m.path).to_string();
    let size_d = media_size(sc, &m).map(human_size).unwrap_or_else(|| "—".into());
    let dims_d = match (m.w, m.h) {
        (Some(w), Some(h)) => format!("{w} by {h} pixels"),
        _ => "—".into(),
    };
    let big = if m.mime.starts_with("image/") {
        format!(r#"<img src="{}" style="max-width:100%;height:auto" alt="">"#, esc(&url))
    } else {
        format!(r#"<p><span class="dashicons {}" style="font-size:64px"></span></p>"#, mime_icon(&m.mime))
    };
    // Canvas editor only for formats canvas can read+write cleanly:
    // jpeg/png/webp get their own codec; gif flattens to a frame, svg would
    // rasterize — both corrupt the original, so no button for those.
    let editor = if matches!(m.mime.as_str(), "image/jpeg" | "image/png" | "image/webp") {
        image_editor(m.id, &esc_js(&url), &s.csrf, !m.backup_path.is_empty())
    } else {
        String::new()
    };
    let notice = if q.get("restored").is_some() {
        r#"<div class="notice notice-success is-dismissible"><p>Image restored from the original.</p></div>"#
    } else if q.get("updated").is_some() {
        r#"<div class="notice notice-success is-dismissible"><p>Media attachment updated.</p></div>"#
    } else {
        ""
    };
    let attached = crud::media_attached_to(&sc.db, m.id).unwrap_or_default();
    let uploaded_to = if attached.is_empty() {
        "(Unattached)".to_string()
    } else {
        attached.iter().map(|(pid, t)| format!(r#"<a href="/admin/edit/{pid}">{}</a>"#, esc(t))).collect::<Vec<_>>().join(", ")
    };
    let inner = format!(
        r##"{chrome}<div class="wrap"><h1 class="wp-heading-inline">Edit Media</h1>
<a href="/admin/media/new" class="page-title-action">Add Media File</a>
<hr class="wp-header-end">
{notice}
<form method="post" action="/admin/media/save">
<input type="hidden" name="csrf" value="{csrf}">
<input type="hidden" name="id" value="{id}">
<div id="poststuff"><div id="post-body" class="metabox-holder columns-2">
<div id="post-body-content" style="position:relative">
<input type="text" name="title" id="title" value="{title}" style="width:100%;font-size:1.7em;padding:6px 8px;margin-bottom:8px" autocomplete="off">
<div id="edit-slug-box" class="hide-if-no-js"><strong>Permalink:</strong> <a href="{abs}" target="_blank">{abs}</a></div>
{editor}
<div class="wp_attachment_image" style="margin:10px 0">{big}</div>
<p><label for="attachment_alt"><strong>Alternative Text</strong></label><br>
<input type="text" name="alt" id="attachment_alt" value="{alt}" class="large-text">
<a href="https://www.w3.org/WAI/tutorials/images/decision-tree/" target="_blank">Learn how to describe the purpose of the image</a>. Leave empty if the image is purely decorative.</p>
<p><label for="attachment_caption"><strong>Image Caption</strong></label><br>
<textarea name="caption" id="attachment_caption" class="large-text" rows="3">{caption}</textarea></p>
<p><label for="attachment_content"><strong>Description</strong></label><br>
<textarea name="description" id="attachment_content" class="large-text" rows="6">{desc}</textarea></p>
</div>
<div id="postbox-container-1" class="postbox-container">
<div class="postbox"><h2 class="hndle"><span>Save</span></h2><div class="inside">
<div class="submitbox" id="submitpost">
<div id="misc-publishing-actions">
<div class="misc-pub-section"><span>Uploaded on <strong>{created}</strong></span></div>
<div class="misc-pub-section"><span>Uploaded by: <strong>{author}</strong></span></div>
<div class="misc-pub-section"><span>Uploaded to: {uploaded_to}</span></div>
<div class="misc-pub-section"><span>File URL:</span><br><input type="text" class="widefat" readonly value="{abs}" onclick="this.select()"></div>
<div class="misc-pub-section"><a href="#" class="button" onclick="navigator.clipboard.writeText('{abs_js}');return false">Copy URL to clipboard</a>
<a href="/admin/media/download/{id}">Download file</a></div>
<div class="misc-pub-section"><span>File name: <strong>{fname}</strong></span><br>
<span>File type: <strong>{mime}</strong></span><br>
<span>File size: <strong>{size}</strong></span><br>
<span>Dimensions: <strong>{dims}</strong></span></div>
</div>
<div id="major-publishing-actions">
<div id="delete-action"><button form="media-del" class="button-link submitdelete deletion" style="background:none;border:none;padding:0;color:#b32d2e;cursor:pointer" onclick="return confirm('Delete permanently?')">Delete permanently</button></div>
<div id="publishing-action"><input type="submit" name="save" id="publish" class="button button-primary button-large" value="Update"></div>
<div class="clear"></div>
</div>
</div></div></div>
</div>
<div class="clear"></div></div></div>
</form>
<form method="post" action="/admin/media/delete" id="media-del"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="id" value="{id}"></form></div>"##,
        chrome = chrome("media", &s),
        notice = notice,
        csrf = s.csrf,
        id = m.id,
        title = esc(&m.title),
        big = big,
        editor = editor,
        alt = esc(&m.alt),
        caption = esc(&m.caption),
        desc = esc(&m.description),
        created = esc(&m.created),
        author = esc(if m.owner.is_empty() { &s.email } else { &m.owner }),
        uploaded_to = uploaded_to,
        abs = esc(&abs),
        abs_js = esc_js(&abs),
        fname = esc(&file_name),
        mime = esc(&m.mime),
        size = size_d,
        dims = dims_d,
    );
    html_page(&s, "Edit Media", &inner, true).into_response()
}

#[derive(Deserialize)]
pub(crate) struct MediaSave {
    csrf: String,
    id: i64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    alt: String,
    #[serde(default)]
    caption: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    ajax: String,
}

pub(crate) async fn media_save(State(app): State<App>, headers: HeaderMap, Form(f): Form<MediaSave>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = check_csrf(&s, &f.csrf) {
        return r;
    }
    match crud::media_update(&sc.db, f.id, &f.title, &f.alt, &f.caption, &f.description) {
        Ok(_) if !f.ajax.is_empty() => "ok".into_response(),
        Ok(_) => Redirect::to(&format!("/admin/media/edit/{}?updated=1", f.id)).into_response(),
        Err(e) => {
            eprintln!("media_save id={}: {e}", f.id);
            (StatusCode::BAD_REQUEST, "save failed").into_response()
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct MediaId {
    csrf: String,
    id: i64,
}

/// Single delete: row + local file + any generated sizes.
pub(crate) async fn media_delete(State(app): State<App>, headers: HeaderMap, Form(f): Form<MediaId>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = check_csrf(&s, &f.csrf) {
        return r;
    }
    match crud::media_delete(&sc.db, f.id) {
        Ok(row) => {
            if let Some((p, sz, bp)) = row { remove_media_files(&sc, &p, &sz, &bp); }
            Redirect::to("/admin/media?deleted=1").into_response()
        }
        Err(e) => {
            eprintln!("media_delete id={}: {e}", f.id);
            (StatusCode::BAD_REQUEST, "delete failed").into_response()
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct MediaDetach {
    csrf: String,
    media_id: i64,
    post_id: i64,
}

pub(crate) async fn media_detach(State(app): State<App>, headers: HeaderMap, Form(f): Form<MediaDetach>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = check_csrf(&s, &f.csrf) {
        return r;
    }
    let _ = crud::media_detach(&sc.db, f.media_id, f.post_id);
    Redirect::to("/admin/media").into_response()
}

/// Bulk action: only `delete` — WP parity with the selector. Serves both the
/// list-view bulk-actions form and the grid-view "Delete Selected" bar; `back`
/// preserves the active filter query string.
pub(crate) async fn media_bulk(State(app): State<App>, headers: HeaderMap, Form(map): Form<HashMap<String, String>>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = check_csrf(&s, map.get("csrf").map(|v| v.as_str()).unwrap_or("")) {
        return r;
    }
    let mut deleted = 0i64;
    if map.get("action").map(|v| v.as_str()) == Some("delete") {
        for (k, _) in map.iter().filter(|(k, _)| k.starts_with("m-")) {
            if let Ok(id) = k[2..].parse::<i64>() {
                if let Ok(Some((p, sz, bp))) = crud::media_delete(&sc.db, id) {
                    remove_media_files(&sc, &p, &sz, &bp);
                    deleted += 1;
                }
            }
        }
    }
    let back = map.get("back").cloned().unwrap_or_else(|| "/admin/media".to_string());
    // Redirect target must stay a local /admin path — an open-redirect to an
    // arbitrary URL via `back` is worse than losing the filter state.
    let back = if back.starts_with("/admin/") { back } else { "/admin/media".to_string() };
    let sep = if back.contains('?') { "&" } else { "?" };
    Redirect::to(&format!("{back}{sep}deleted={deleted}")).into_response()
}

/// Force-download with Content-Disposition (WP "Download file" plugin action).
pub(crate) async fn media_download(State(app): State<App>, headers: HeaderMap, Path(id): Path<i64>) -> Response {
    let Ok((_, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    let Some(m) = crud::media_get(&sc.db, id).ok().flatten() else {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    };
    // Path comes from the DB — guard traversal even though readers already do.
    if m.path.contains("..") || m.path.starts_with('/') {
        return (StatusCode::BAD_REQUEST, "bad path").into_response();
    }
    let p = sc.www.join("wp-content/uploads").join(&m.path);
    let Ok(b) = std::fs::read(&p) else {
        // File lives on the origin host — send the browser there.
        let origin = site_url(&sc).trim_end_matches('/').to_string();
        return Redirect::temporary(&format!("{origin}/wp-content/uploads/{}", m.path)).into_response();
    };
    let raw = m.path.rsplit('/').next().unwrap_or("file");
    // Strip header-breaking bytes: quotes, CRLF, control chars.
    let name: String = raw.chars().filter(|c| *c != '"' && !c.is_control()).collect();
    (
        [
            (header::CONTENT_TYPE, m.mime.clone()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{name}\"")),
        ],
        b,
    )
        .into_response()
}

/// Delete the stored file plus every generated size listed in sizes_json and
/// the Edit-Image backup. Only files the DB explicitly owns are removed — no
/// name-pattern globbing, so a legitimately-named `foo-300x200.png` attachment
/// is never collateral.
fn remove_media_files(sc: &SiteCtx, path: &str, sizes_json: &str, backup_path: &str) {
    if path.contains("..") || path.starts_with('/') {
        return;
    }
    let base = sc.www.join("wp-content/uploads");
    let _ = std::fs::remove_file(base.join(path));
    if !backup_path.is_empty() && !backup_path.contains("..") && !backup_path.starts_with('/') {
        let _ = std::fs::remove_file(base.join(backup_path));
    }
    let Some(dir) = path.rsplit_once('/').map(|(d, _)| d.to_string()) else { return };
    if dir.contains("..") {
        return;
    }
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(sizes_json) {
        if let Some(map) = v.as_object() {
            for sz in map.values() {
                if let Some(f) = sz.get("file").and_then(|f| f.as_str()) {
                    if !f.contains("..") && !f.contains('/') {
                        let _ = std::fs::remove_file(base.join(&dir).join(f));
                    }
                }
            }
        }
    }
}

/// File size: local stat first; for remote-only media (imported, file lives
/// on the WP origin) fall back to the largest filesize recorded in sizes_json.
fn media_size(sc: &SiteCtx, m: &crud::MediaRow) -> Option<u64> {
    if let Ok(f) = std::fs::metadata(sc.www.join("wp-content/uploads").join(&m.path)) {
        return Some(f.len());
    }
    serde_json::from_str::<serde_json::Value>(&m.sizes_json)
        .ok()
        .and_then(|v| {
            v.as_object()?
                .values()
                .filter_map(|sz| sz.get("filesize").and_then(|n| n.as_u64()))
                .max()
        })
}


/// Escape for embedding inside a single-quoted JS string in an inline handler:
/// esc() covers HTML context; add `'` → \' so an apostrophe can't break out.
fn esc_js(s: &str) -> String {
    esc(s).replace('\\', "\\\\").replace('\'', "\\'")
}

fn mime_icon(mime: &str) -> &'static str {
    match mime.split('/').next().unwrap_or("") {
        "image" => "dashicons-format-image",
        "audio" => "dashicons-format-audio",
        "video" => "dashicons-format-video",
        _ => match mime {
            "application/pdf" => "dashicons-pdf",
            "font/woff" | "font/woff2" | "font/ttf" => "dashicons-editor-textcolor",
            _ => "dashicons-media-default",
        },
    }
}

fn human_size(b: u64) -> String {
    if b >= 1_048_576 {
        format!("{:.2} MB", b as f64 / 1_048_576.0)
    } else if b >= 1024 {
        format!("{:.2} KB", b as f64 / 1024.0)
    } else {
        format!("{b} B")
    }
}

pub(crate) async fn media_upload(State(app): State<App>, headers: HeaderMap, mut multipart: axum::extract::Multipart) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    let mut csrf = String::new();
    let mut alt = String::new();
    let mut file_name = String::new();
    let mut bytes = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "csrf" {
            csrf = field.text().await.unwrap_or_default();
        } else if name == "alt" {
            alt = field.text().await.unwrap_or_default();
        } else if name == "file" {
            file_name = field.file_name().unwrap_or("upload.bin").to_string();
            bytes = field.bytes().await.unwrap_or_default().to_vec();
        }
    }
    if let Err(r) = check_csrf(&s, &csrf) {
        return r;
    }
    if bytes.is_empty() {
        return (StatusCode::BAD_REQUEST, "empty file").into_response();
    }
    // Extension allowlist (A4): scriptable types (svg/html/xml/…) rejected
    // before save_upload's defense-in-depth re-check.
    if !crud::upload_ext_ok(&file_name) {
        return (StatusCode::UNSUPPORTED_MEDIA_TYPE, "file type not allowed").into_response();
    }
    let uploads = sc.www.join("wp-content/uploads");
    match crud::save_upload(&sc.db, &uploads, &file_name, &bytes, &alt, &s.email) {
        Ok(id) => match headers.get("x-requested-with").and_then(|v| v.to_str().ok()) {
            Some("XMLHttpRequest") => id.to_string().into_response(),
            _ => Redirect::to("/admin/media").into_response(),
        },
        Err(e) => {
            eprintln!("media_upload: {e}");
            (StatusCode::BAD_REQUEST, "upload failed").into_response()
        }
    }
}

pub(crate) fn intake_headers(allowed: &[String], origin: Option<&str>, body: &'static str, code: StatusCode) -> Response {
    let mut r = Response::builder().status(code);
    if let Some(o) = origin {
        if intake::cors_ok(allowed, Some(o)) {
            r = r.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, o);
        }
    }
    r.header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from(body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

pub(crate) async fn intake_form(State(app): State<App>, ConnectInfo(peer): ConnectInfo<SocketAddr>, headers: HeaderMap, Path(slug): Path<String>, Form(fields): Form<HashMap<String, String>>) -> Response {
    let sc = site_for(&app, &headers);
    intake_post(&app, &headers, Some(peer), &allowed_origins(sc), None, |ip| intake::submit_form(&sc.db, &slug, &fields, ip))
}

pub(crate) async fn intake_comment(State(app): State<App>, ConnectInfo(peer): ConnectInfo<SocketAddr>, headers: HeaderMap, Form(fields): Form<HashMap<String, String>>) -> Response {
    let sc = site_for(&app, &headers);
    // A static page can't flash a status inline — the intake redirects back
    // to the post with a fragment the comment block's :target divs style.
    let back = fields.get("_page").map(|s| s.as_str());
    intake_post(&app, &headers, Some(peer), &allowed_origins(sc), back, |ip| {
        intake::submit_comment(&sc.db, &fields, ip).map(|(result, pid)| {
            if pid > 0 && matches!(result, intake::CommentResult::Submitted) {
                regen_posts(sc, &[pid]);
            }
            match result {
                intake::CommentResult::Submitted => "comment-submitted",
                intake::CommentResult::Held => "comment-held",
                intake::CommentResult::Closed => "comment-closed",
            }
        })
    })
}

pub(crate) async fn intake_review(State(app): State<App>, ConnectInfo(peer): ConnectInfo<SocketAddr>, headers: HeaderMap, Form(fields): Form<HashMap<String, String>>) -> Response {
    let sc = site_for(&app, &headers);
    let back = fields.get("_page").map(|s| s.as_str());
    intake_post(&app, &headers, Some(peer), &allowed_origins(sc), back, |ip| intake::submit_review(&sc.db, &fields, ip))
}

/// Only same-site paths may become the post-POST redirect target: an
/// attacker-supplied `_page` like `https://evil/` must not bounce users.
/// Empty → "/" (the form still lands somewhere sane).
fn safe_back(page: Option<&str>) -> String {
    match page {
        Some(p) if p.starts_with('/') && !p.starts_with("//") => p.to_string(),
        _ => "/".to_string(),
    }
}

/// Re-render the given posts' files in the background — the
/// approve→republish hook (phase 4.4). This is THE regen funnel: every
/// mutation path (post save, quick/bulk edit, trash/restore, comment
/// transitions, contact/review submits) funnels through here, so
/// dependent-artifact invalidation lives inside it — callers get term
/// archives, homepage, feeds and sitemap invalidation by construction.
/// Cheap (one file per post) and runs off the request thread.
///
/// Ordering: callers invoke regen_posts AFTER their DB writes commit, and
/// this runs on a blocking task — invalidation therefore never races an
/// open transaction. Residual delete-vs-late-write window: a request that
/// started rendering before the first sweep can re-cache stale output
/// after the delete. The post-regen second sweep below shrinks that
/// window to ≈ one render's duration; closing it fully needs a per-path
/// render lock — deliberately deferred (audit phase 5).
pub(crate) fn regen_posts(sc: &SiteCtx, ids: &[i64]) {
    regen_posts_with_terms(sc, ids, &[]);
}

/// `regen_posts` for save paths that REPLACE a post's term assignments
/// (quick edit, full save): `extra_terms` carries the pre-commit
/// (taxonomy, slug) snapshot per affected post so archives the post just
/// LEFT get invalidated too — current post_terms only sees the new set.
pub(crate) fn regen_posts_with_terms(sc: &SiteCtx, ids: &[i64], extra_terms: &[(i64, String, String)]) {
    let mut ids: Vec<i64> = ids.iter().copied().filter(|i| *i > 0).collect();
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return;
    }
    let mut terms_by_post: HashMap<i64, Vec<(String, String)>> = HashMap::new();
    for (pid, tax, slug) in extra_terms {
        terms_by_post
            .entry(*pid)
            .or_default()
            .push((tax.clone(), slug.clone()));
    }
    let db = sc.db.clone();
    let templates = sc.templates.clone();
    let out = sc.out.clone();
    let tour_base = crate::admin::tour_base(sc).to_string();
    tokio::task::spawn_blocking(move || {
        let conn = match crate::db::conn(&db) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("regen conn: {e:#}");
                return;
            }
        };
        for id in &ids {
            let extra = terms_by_post.get(id).cloned().unwrap_or_default();
            generate::invalidate_dependents(&conn, &out, *id, &tour_base, &extra);
            if let Err(e) = generate::regen_post(&db, &templates, &out, *id, &tour_base) {
                eprintln!("regen post {id}: {e:#}");
            }
        }
        // Second sweep: delete anything a racing in-flight miss re-cached
        // during the regen above (see the residual-window note).
        for id in &ids {
            let extra = terms_by_post.get(id).cloned().unwrap_or_default();
            generate::invalidate_dependents(&conn, &out, *id, &tour_base, &extra);
        }
        // Re-emit the site files the sweep invalidated — feeds and
        // sitemap*.xml have no lazy re-render path (pure file reads), so
        // deleting them without a rewrite would 404 them until the next
        // rebuild. Pure SQL→string, cheap.
        let base = crate::db::setting_value(&db, "site_url", "/");
        if let Err(e) = crate::sitefx::emit_site_files(&db, &templates, &out, &base, &tour_base) {
            eprintln!("emit_site_files: {e:#}");
        }
    });
}
/// Drop every regenerated file in the cache dir (HTML/md/xml/txt/json) but
/// keep user files (uploads/media) and `www/` — everything deleted here
/// re-renders lazily on the next request. Called after changes that affect
/// *every* page: menu/widget/settings/taxonomy saves.
pub(crate) fn bust_cache(sc: &SiteCtx) {
    sc.settings_bust();
    bust_cache_dir(&sc.out);
}

/// Sweep generated files under `dir` — callable from contexts that have a
/// path but no SiteCtx (e.g. the rebuild job).
pub(crate) fn bust_cache_dir(dir: &std::path::Path) {
    // Template environments are process-cached per templates dir; a bust is
    // the admin signal that theme/template content may have changed, so the
    // cached Environment must go too or editors see stale templates.
    crate::render::env_cache_bust();
    // Site_data is (site_url, lang)-keyed and holds post/tour rows — a full
    // bust means widgets/menus/settings changed; the cache must go too.
    crate::render::site_data_bust();
    fn sweep(dir: &std::path::Path) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name();
            let name = name.to_string_lossy();
            if p.is_dir() {
                if name == "wp-content" || name.starts_with('.') {
                    continue;
                }
                sweep(&p);
                let _ = std::fs::remove_dir(&p); // only removes when empty
            } else {
                // Cache files + atomic-write tmp orphans: write_cache's
                // "{stem}.{pid}.{n}.tmp" and generate::atomic_write's
                // "tmp-{pid}-{stamp}" suffixes must not survive a bust.
                let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("");
                if matches!(ext, "html" | "htm" | "md" | "xml" | "txt" | "json" | "tmp")
                    || name.contains(".tmp-")
                {
                    let _ = std::fs::remove_file(&p);
                }
            }
        }
    }
    sweep(dir);
}

pub(crate) fn intake_post(app: &App, headers: &HeaderMap, peer: Option<SocketAddr>, allowed: &[String], back: Option<&str>, f: impl FnOnce(&str) -> anyhow::Result<&'static str>) -> Response {
    let origin = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok());
    if !intake::cors_ok(allowed, origin) {
        return intake_headers(allowed, origin, "forbidden", StatusCode::FORBIDDEN);
    }
    if headers.get(header::CONTENT_LENGTH).and_then(|v| v.to_str().ok()).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0) > 32 * 1024 {
        return intake_headers(allowed, origin, "too large", StatusCode::PAYLOAD_TOO_LARGE);
    }
    let ip = client_ip(headers, peer);
    if !app.intake_rate.lock().allow(&ip, 10, Duration::from_secs(300)) {
        return intake_headers(allowed, origin, "rate", StatusCode::TOO_MANY_REQUESTS);
    }
    match f(&ip) {
        Ok(msg) => {
            if let Some(page) = back {
                // `msg` doubles as the fragment: comment-submitted /
                // comment-held / comment-closed / comment-error match the
                // :target status divs in comments_wpdiscuz.html.
                return Redirect::to(&format!("{}#{}", safe_back(Some(page)), msg)).into_response();
            }
            intake_headers(allowed, origin, msg, StatusCode::OK)
        }
        Err(_) => {
            if let Some(page) = back {
                return Redirect::to(&format!("{}#comment-error", safe_back(Some(page)))).into_response();
            }
            intake_headers(allowed, origin, "bad", StatusCode::BAD_REQUEST)
        }
    }
}


/// JSON detail for the grid-view attachment modal (mini `wp_ajax_get-attachment`).
/// Includes prev/next ids within the current page's `ids` param.
pub(crate) async fn media_json(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Ok((_s, sc)) = need_auth(&app, &headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Some(m) = crud::media_get(&sc.db, id).ok().flatten() else {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    };
    let url = format!("/wp-content/uploads/{}", m.path);
    let abs = format!("{}{}", site_url(&sc).trim_end_matches('/'), url);
    let thumb = thumb_url(&abs, &m);
    let ids: Vec<i64> = q
        .get("ids")
        .map(|v| v.split(',').filter_map(|x| x.parse().ok()).collect())
        .unwrap_or_default();
    let pos = ids.iter().position(|x| *x == id);
    let prev = pos.filter(|&i| i > 0).map(|i| ids[i - 1]);
    let next = pos.filter(|&i| i + 1 < ids.len()).map(|i| ids[i + 1]);
    let file_name = m.path.rsplit('/').next().unwrap_or(&m.path);
    let attached = crud::media_attached_to(&sc.db, m.id).unwrap_or_default();
    let uploaded_to: Vec<serde_json::Value> = attached
        .iter()
        .map(|(pid, t)| serde_json::json!({"id": pid, "title": t}))
        .collect();
    let size_d = media_size(sc, &m).map(human_size).unwrap_or_default();
    let dims = match (m.w, m.h) {
        (Some(w), Some(h)) => format!("{w}×{h}"),
        _ => String::new(),
    };
    let j = serde_json::json!({
        "id": m.id, "url": abs, "thumb": thumb, "path": m.path, "file_name": file_name,
        "title": m.title, "alt": m.alt, "caption": m.caption, "description": m.description,
        "mime": m.mime, "w": m.w, "h": m.h, "dims": dims, "size": size_d,
        "created": m.created, "uploaded_to": uploaded_to,
        "edit_url": format!("/admin/media/edit/{}", m.id),
        "prev": prev, "next": next,
    });
    ([(header::CONTENT_TYPE, "application/json")], j.to_string()).into_response()
}

/// Thumbnail for pickers/grid: prefer WP "thumbnail" then "medium" from
/// sizes_json (same dir as the full file); else the full URL.
pub(crate) fn thumb_url(abs_url: &str, m: &crud::MediaRow) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&m.sizes_json) {
        if let Some(map) = v.as_object() {
            for key in ["thumbnail", "medium"] {
                if let Some(f) = map.get(key).and_then(|s| s.get("file")).and_then(|f| f.as_str()) {
                    if !f.contains("..") && !f.contains('/') {
                        if let Some(dir_end) = abs_url.rfind('/') {
                            return format!("{}/{}", &abs_url[..dir_end], f);
                        }
                    }
                }
            }
        }
    }
    abs_url.to_string()
}

/// Item list for the media-frame picker — same filters as the library list
/// page (`mime`, `m`, `s`, `page`), lightweight fields only.
pub(crate) async fn media_items(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Ok((_s, sc)) = need_auth(&app, &headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let mime = q.get("mime").map(|v| v.as_str()).unwrap_or("");
    let month = q.get("m").map(|v| v.as_str()).unwrap_or("");
    let search = q.get("s").map(|v| v.as_str()).unwrap_or("");
    let page: i64 = q.get("page").and_then(|v| v.parse().ok()).unwrap_or(1).clamp(1, 4_000_000_000);
    let per = 40i64;
    let Ok((rows, total)) = crud::media_list(&sc.db, mime, month, search, "date", (page - 1) * per, per) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "db").into_response();
    };
    let origin = site_url(&sc);
    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|m| {
            let url = format!("{}/wp-content/uploads/{}", origin.trim_end_matches('/'), m.path);
            serde_json::json!({
                "id": m.id, "url": url, "thumb": thumb_url(&url, m),
                "title": m.title, "mime": m.mime,
                "file_name": m.path.rsplit('/').next().unwrap_or(&m.path),
            })
        })
        .collect();
    let months = crud::media_months(&sc.db).unwrap_or_default();
    let j = serde_json::json!({"items": items, "total": total, "page": page, "per": per, "months": months});
    ([(header::CONTENT_TYPE, "application/json")], j.to_string()).into_response()
}

/// Canvas image-editor save: replaces the file bytes, keeps id/path.
/// Field `restore=1` (no file) instead copies `backup_path` back over the
/// live file — the "Restore Original" button.
pub(crate) async fn media_replace(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    mut multipart: axum::extract::Multipart,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let mut csrf = String::new();
    let mut restore = false;
    let mut bytes = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name().unwrap_or("") {
            "csrf" => csrf = field.text().await.unwrap_or_default(),
            "restore" => restore = !field.text().await.unwrap_or_default().is_empty(),
            "file" => bytes = field.bytes().await.unwrap_or_default().to_vec(),
            _ => {}
        }
    }
    if let Err(r) = check_csrf(&s, &csrf) {
        return r;
    }
    let uploads = sc.www.join("wp-content/uploads");
    if restore {
        return match crud::media_restore(&sc.db, &uploads, id) {
            Ok(_) => "restored".into_response(),
            Err(e) => (StatusCode::BAD_REQUEST, format!("{e}")).into_response(),
        };
    }
    if bytes.is_empty() {
        return (StatusCode::BAD_REQUEST, "empty").into_response();
    }
    match crud::media_replace_upload(&sc.db, &uploads, id, &bytes) {
        Ok(_) => "saved".into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, format!("{e}")).into_response(),
    }
}

/// Attachment modal for grid mode — WP `wp.media` dialog, lite version:
/// preview left, meta form right, prev/next, Save/Delete/Edit.
fn grid_modal(rows: &[crud::MediaRow], csrf: &str) -> String {
    let ids: Vec<String> = rows.iter().map(|r| r.id.to_string()).collect();
    format!(
        r##"<div id="att-modal" style="display:none;position:fixed;inset:0;z-index:10000;background:rgba(0,0,0,.7)">
<div style="position:absolute;inset:40px;background:#fff;display:flex;flex-direction:column">
<div class="media-frame-title" style="padding:0 16px;border-bottom:1px solid #dcdcde"><h1 style="padding:0;line-height:50px;font-size:22px">Attachment Details</h1></div>
<div style="flex:1;display:flex;min-height:0">
<div id="am-preview" style="flex:1;display:flex;align-items:center;justify-content:center;background:#f0f0f1;overflow:hidden;padding:16px"></div>
<div id="am-side" style="width:300px;border-left:1px solid #dcdcde;padding:16px;overflow:auto">
<p style="margin:0"><strong id="am-title-d"></strong><br><span id="am-meta" style="color:#646970;font-size:12px"></span></p>
<p><a id="am-edit" href="" class="button">Edit Media</a></p>
<form id="am-form">
<input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="ajax" value="1"><input type="hidden" name="id" id="am-id">
<p><label>Title<br><input type="text" name="title" id="am-title" class="widefat"></label></p>
<p><label>Alternative Text<br><input type="text" name="alt" id="am-alt" class="widefat"></label></p>
<p><label>Caption<br><textarea name="caption" id="am-cap" class="widefat" rows="3"></textarea></label></p>
<p><label>Description<br><textarea name="description" id="am-desc" class="widefat" rows="4"></textarea></label></p>
<p id="am-up"></p>
<p><button type="submit" class="button button-primary">Save</button> <span id="am-saved" style="color:#00a32a"></span></p>
</form></div>
</div>
<div class="media-frame-toolbar" style="border-top:1px solid #dcdcde;padding:10px 16px;text-align:right">
<button class="button" id="am-prev">&#8249; Prev</button>
<button class="button" id="am-next">Next &#8250;</button>
<button class="button button-primary" id="am-close">Close</button>
</div>
</div></div>
<script>
(function(){{
var ids=[{ids}],cur=-1,mdl=document.getElementById('att-modal');
document.querySelectorAll('#att-grid .attachment').forEach(function(li){{
  li.addEventListener('click',function(){{if(window._attBulk)return;open(+li.dataset.id);}});
}});
function open(id){{cur=ids.indexOf(id);show(id);mdl.style.display='block';}}
// Exposed for list-mode row Edit links — same modal, same data path.
window.att_open=open;
function show(id){{
  document.getElementById('am-saved').textContent='';
  fetch('/admin/media/json/'+id+'?ids='+ids.join(',')).then(r=>r.json()).then(d=>{{
    document.getElementById('am-id').value=d.id;
    document.getElementById('am-title-d').textContent=d.title||d.file_name;
    var pv=document.getElementById('am-preview');pv.textContent='';
    if(d.mime.indexOf('image/')===0){{var im=document.createElement('img');im.src=d.url;im.style.cssText='max-width:100%;max-height:100%;object-fit:contain';pv.appendChild(im);}}
    else{{var sp=document.createElement('span');sp.className='dashicons dashicons-media-default';sp.style.fontSize='96px';pv.appendChild(sp);}}
    var meta=[d.file_name,d.created,d.size,d.dims,d.mime].filter(Boolean).join(' · ');
    document.getElementById('am-meta').textContent=meta;
    document.getElementById('am-edit').href=d.edit_url;
    document.getElementById('am-title').value=d.title;
    document.getElementById('am-alt').value=d.alt;
    document.getElementById('am-cap').value=d.caption;
    document.getElementById('am-desc').value=d.description;
    var up=document.getElementById('am-up');up.textContent='';
    if(d.uploaded_to&&d.uploaded_to.length){{
      up.appendChild(document.createTextNode('Uploaded to: '));
      d.uploaded_to.forEach(function(u,i){{
        if(i)up.appendChild(document.createTextNode(', '));
        var a=document.createElement('a');a.href='/admin/edit/'+u.id;a.textContent=u.title;up.appendChild(a);
      }});
    }}else{{up.textContent='(Unattached)';}}
    document.getElementById('am-prev').disabled=d.prev==null;
    document.getElementById('am-next').disabled=d.next==null;
    window._amp=d.prev;window._amn=d.next;
  }});
}}
document.getElementById('am-prev').addEventListener('click',function(){{if(window._amp)show(window._amp);}});
document.getElementById('am-next').addEventListener('click',function(){{if(window._amn)show(window._amn);}});
document.getElementById('am-close').addEventListener('click',function(){{mdl.style.display='none';}});
mdl.addEventListener('click',function(e){{if(e.target===mdl)mdl.style.display='none';}});
document.getElementById('am-form').addEventListener('submit',function(e){{
  e.preventDefault();
  fetch('/admin/media/save',{{method:'POST',body:new URLSearchParams(new FormData(e.target))}}).then(function(r){{
    document.getElementById('am-saved').textContent=r.ok?'Saved.':'Save failed ('+r.status+')';
    if(r.ok)document.getElementById('am-title-d').textContent=document.getElementById('am-title').value;
  }});
}});
}})();
</script>"##,
        csrf = csrf,
        ids = ids.join(","),
    )
}

/// WP `wp.media` frame for choosing one image — used by the Featured image
/// metabox on post editors. Tabs: Upload files (XHR multi) + Media Library
/// (filter/search/infinite scroll). On select: `mf_select(id, thumbUrl)` —
/// callers provide it, else it updates #featured-id/#featured-thumb.
pub(crate) fn media_frame_html(csrf: &str) -> String {
    format!(
        r##"<div id="mf-frame" style="display:none;position:fixed;inset:0;z-index:100100;background:rgba(0,0,0,.7)">
<div style="position:absolute;inset:30px;background:#fff;display:flex;flex-direction:column;box-shadow:0 5px 15px rgba(0,0,0,.7)">
<div class="media-frame-title" style="padding:0 16px;border-bottom:1px solid #dcdcde;display:flex;align-items:center">
<h1 style="padding:0;line-height:50px;font-size:22px;margin-right:auto">Featured image</h1>
<a href="#" style="font-size:24px;text-decoration:none;color:#646970" onclick="mf_close();return false">&times;</a>
</div>
<div class="media-frame-menu" style="border-bottom:1px solid #dcdcde;padding:0 16px">
<a href="#" class="media-menu-item active" id="mf-t-lib" style="display:inline-block;padding:12px 16px 12px 0;border-bottom:2px solid #2271b1" onclick="mf_tab('lib');return false">Media Library</a>
<a href="#" class="media-menu-item" id="mf-t-up" style="display:inline-block;padding:12px 16px;color:#2271b1" onclick="mf_tab('up');return false">Upload files</a>
</div>
<div id="mf-pane-up" style="display:none;flex:1;overflow:auto;padding:30px;text-align:center">
<div id="mf-drop" style="border:4px dashed #c3c4c7;min-height:220px;display:flex;align-items:center;justify-content:center;flex-direction:column">
<p style="font-size:20px;color:#3c434a">Drop files to upload</p><p>or</p>
<p><button type="button" class="button button-hero" onclick="document.getElementById('mf-file').click()">Select Files</button></p>
<p style="color:#646970">Maximum upload file size: {upload_limit}.</p>
</div>
<input type="file" id="mf-file" multiple style="display:none">
<div id="mf-prog"></div>
</div>
<div id="mf-pane-lib" style="flex:1;display:flex;min-height:0">
<div style="flex:1;display:flex;flex-direction:column;min-width:0">
<div style="padding:10px 16px;border-bottom:1px solid #e0e0e0;display:flex;gap:8px;align-items:center">
<select id="mf-mime"><option value="image" selected>Images</option><option value="video">Video</option><option value="audio">Audio</option><option value="application">Documents</option><option value="">All media items</option></select>
<select id="mf-month"><option value="">All dates</option></select>
<input type="search" id="mf-search" placeholder="Search media" style="margin-left:auto;min-width:200px">
</div>
<div id="mf-grid" style="flex:1;overflow:auto;padding:8px;display:flex;flex-wrap:wrap;align-content:flex-start;gap:4px"></div>
<div id="mf-more-w" style="text-align:center;padding:8px;display:none"><button class="button" id="mf-more">Load more</button></div>
</div>
<div id="mf-side" style="width:290px;border-left:1px solid #dcdcde;overflow:auto;padding:14px;display:none">
<h2 style="font-size:13px;text-transform:uppercase;color:#646970;margin:0 0 10px">Attachment Details</h2>
<div id="mf-side-body"></div>
</div>
</div>
<div class="media-frame-toolbar" style="border-top:1px solid #dcdcde;padding:10px 16px;text-align:right">
<button class="button button-primary button-large" id="mf-set" disabled>Set featured image</button>
</div>
</div></div>
<script>
var mf_sel=null,mf_page=1,mf_total=0,mf_loading=false;
var mf_mode='featured';
function mf_open(mode){{
  mf_mode=mode||'featured';
  document.getElementById('mf-set').textContent=mf_mode==='insert'?'Insert into post':'Set featured image';
  document.getElementById('mf-frame').style.display='block';
  mf_tab('lib');
  if(!mf_loaded)mf_load(1,true);
}}
function mf_close(){{document.getElementById('mf-frame').style.display='none';}}
function mf_tab(t){{
  var lib=t==='lib';
  document.getElementById('mf-pane-lib').style.display=lib?'flex':'none';
  document.getElementById('mf-pane-up').style.display=lib?'none':'block';
  var a=document.getElementById('mf-t-lib'),b=document.getElementById('mf-t-up');
  a.style.borderBottom=lib?'2px solid #2271b1':'none';a.style.color=lib?'#1d2327':'#2271b1';
  b.style.borderBottom=lib?'none':'2px solid #2271b1';b.style.color=lib?'#2271b1':'#1d2327';
}}
var mf_loaded=false;
function mf_qs(){{
  var p=new URLSearchParams();
  p.set('mime',document.getElementById('mf-mime').value);
  var m=document.getElementById('mf-month').value;if(m)p.set('m',m);
  var s=document.getElementById('mf-search').value;if(s)p.set('s',s);
  return p.toString();
}}
function mf_load(page,reset){{
  if(mf_loading)return;mf_loading=true;
  fetch('/admin/media/items?'+mf_qs()+'&page='+page).then(r=>r.json()).then(d=>{{
    mf_loading=false;mf_loaded=true;mf_page=d.page;mf_total=d.total;
    if(d.months&&!window._mf_months){{
      window._mf_months=1;
      var mo=document.getElementById('mf-month');
      d.months.forEach(function(m){{var o=document.createElement('option');o.value=m;o.textContent=m;mo.appendChild(o);}});
    }}
    var g=document.getElementById('mf-grid');
    if(reset)g.textContent='';
    if(!d.items.length&&!g.children.length)g.innerHTML='<p style="padding:20px;color:#646970">No media files found.</p>';
    d.items.forEach(function(it){{
      var li=document.createElement('div');
      li.className='mf-item';li.dataset.id=it.id;
      li.style.cssText='width:112px;height:112px;border:2px solid transparent;cursor:pointer;position:relative;overflow:hidden;background:#f0f0f1';
      if(it.mime.indexOf('image/')===0){{
        var im=document.createElement('img');im.src=it.thumb;im.loading='lazy';
        im.style.cssText='width:100%;height:100%;object-fit:cover;display:block';
        li.appendChild(im);
      }}else{{
        var sp=document.createElement('span');sp.className='dashicons dashicons-media-default';
        sp.style.cssText='font-size:42px;line-height:112px;text-align:center;display:block';li.appendChild(sp);
      }}
      var cap=document.createElement('div');cap.textContent=it.title||it.file_name;
      cap.style.cssText='position:absolute;left:0;right:0;bottom:0;background:rgba(255,255,255,.92);font-size:11px;padding:2px 4px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis';
      li.appendChild(cap);
      li.addEventListener('click',function(){{mf_pick(it.id,li);}});
      li.addEventListener('dblclick',function(){{mf_pick(it.id,li);document.getElementById('mf-set').click();}});
      g.appendChild(li);
    }});
    document.getElementById('mf-more-w').style.display=(mf_page*d.per<mf_total)?'block':'none';
  }}).catch(function(){{mf_loading=false;}});
}}
function mf_pick(id,el){{
  mf_sel=id;
  document.querySelectorAll('.mf-item').forEach(function(x){{x.style.borderColor='transparent';x.querySelector('.mf-check')&&x.querySelector('.mf-check').remove();}});
  el.style.borderColor='#2271b1';
  var ck=document.createElement('span');ck.className='mf-check';ck.textContent='✓';
  ck.style.cssText='position:absolute;top:0;right:0;background:#2271b1;color:#fff;width:24px;height:24px;line-height:24px;text-align:center;font-weight:bold';
  el.appendChild(ck);
  document.getElementById('mf-set').disabled=false;
  var side=document.getElementById('mf-side');side.style.display='block';
  fetch('/admin/media/json/'+id).then(r=>r.json()).then(d=>{{
    var b=document.getElementById('mf-side-body');b.textContent='';
    if(d.mime.indexOf('image/')===0){{var im=document.createElement('img');im.src=d.thumb||d.url;im.style.cssText='max-width:100%;height:auto;display:block;margin-bottom:8px';b.appendChild(im);}}
    var p=document.createElement('p');p.style.margin='0 0 8px';
    var st=document.createElement('strong');st.textContent=d.file_name;p.appendChild(st);
    p.appendChild(document.createElement('br'));
    var meta=document.createElement('span');meta.style.cssText='color:#646970;font-size:12px';
    meta.textContent=[d.created,d.size,d.dims,d.mime].filter(Boolean).join(' · ');p.appendChild(meta);
    b.appendChild(p);
    var e=document.createElement('a');e.href=d.edit_url;e.textContent='Edit Image';e.style.display='block';b.appendChild(e);
  }});
}}
document.getElementById('mf-set').addEventListener('click',function(){{
  if(!mf_sel)return;
  fetch('/admin/media/json/'+mf_sel).then(r=>r.json()).then(function(d){{
    if(mf_mode==='insert'){{
      var img='<img src="'+(d.url||d.thumb)+'" alt="'+(d.alt||'')+'" />';
      var ed=(typeof tinymce!=='undefined'&&tinymce.get('content'))?tinymce.get('content'):null;
      if(ed&&!ed.hidden){{ed.execCommand('mceInsertContent',false,img);}}
      else{{var ta=document.getElementById('content');if(ta)ta.value+=img;}}
    }}else{{
      mf_select(d.id,d.thumb||d.url);
    }}
    mf_close();
  }});
}});
document.getElementById('mf-more').addEventListener('click',function(){{mf_load(mf_page+1,false);}});
document.getElementById('mf-grid').addEventListener('scroll',function(e){{
  var el=e.target;
  if(el.scrollTop+el.clientHeight>el.scrollHeight-200&&mf_page*40<mf_total)mf_load(mf_page+1,false);
}});
['mf-mime','mf-month'].forEach(function(i){{document.getElementById(i).addEventListener('change',function(){{mf_load(1,true);}});}});
document.getElementById('mf-search').addEventListener('input',function(){{clearTimeout(window._mfs);window._mfs=setTimeout(function(){{mf_load(1,true);}},300);}});
document.getElementById('mf-frame').addEventListener('click',function(e){{if(e.target.id==='mf-frame')mf_close();}});
document.getElementById('mf-file').addEventListener('change',function(){{mf_upload(this.files);this.value='';}});
var mfd=document.getElementById('mf-drop');
mfd.addEventListener('dragover',function(e){{e.preventDefault();mfd.style.borderColor='#2271b1';}});
mfd.addEventListener('dragleave',function(){{mfd.style.borderColor='#c3c4c7';}});
mfd.addEventListener('drop',function(e){{e.preventDefault();mfd.style.borderColor='#c3c4c7';mf_upload(e.dataTransfer.files);}});
function mf_upload(files){{
  var q=[].slice.call(files);
  function next(){{
    if(!q.length){{mf_tab('lib');mf_load(1,true);return;}}
    var f=q.shift();
    var fd=new FormData();fd.append('csrf','{csrf}');fd.append('file',f);
    var div=document.createElement('div');div.textContent=f.name+' …';
    document.getElementById('mf-prog').appendChild(div);
    var x=new XMLHttpRequest();x.open('POST','/admin/media/upload');
    x.onload=function(){{div.textContent=f.name+(x.status<300?' — uploaded':' — error');next();}};
    x.send(fd);
  }}
  next();
}}
// default select handler — page code may override window.mf_select
window.mf_select = window.mf_select || function(id,thumb){{
  document.getElementById('featured-id').value=id;
  var w=document.getElementById('featured-thumb-wrap');
  var t=document.getElementById('featured-thumb');t.textContent='';
  var im=document.createElement('img');im.src=thumb;im.style.cssText='max-width:100%;height:auto;display:block';
  t.appendChild(im);
  w.style.display='';document.getElementById('featured-hint').style.display='';
  document.getElementById('featured-remove').style.display='';
  document.getElementById('set-featured-image').style.display='none';
}};
</script>"##,
        csrf = csrf,
        upload_limit = crate::db::UPLOAD_LIMIT_DISPLAY,
    )
}

/// WP image-editor markup + canvas logic for the Edit Media page.
/// Crop via drag (optional aspect lock), rotate/flip transform buttons,
/// scale via W×H inputs with aspect lock, Save = canvas.toBlob →
/// /admin/media/replace/{id}. `has_backup` shows "Restore Original" — the
/// server kept `media/{id}_original.{ext}` on first save.
fn image_editor(id: i64, url: &str, csrf: &str, has_backup: bool) -> String {
    let restore_btn = if has_backup {
        r#"<button type="button" class="button" id="ie-restore" style="margin-left:12px">Restore Original</button>"#
    } else {
        ""
    };
    format!(
        r##"<div style="margin:10px 0"><a href="#imgedit" class="button" id="ie-open">Edit Image</a></div>
<div id="ie-panel" style="display:none;border:1px solid #dcdcde;padding:12px;margin:10px 0;background:#f9f9f9">
<div style="margin-bottom:8px;display:flex;gap:8px;flex-wrap:wrap;align-items:center">
<button type="button" class="button" id="ie-crop">Crop</button>
<label style="font-size:12px"><input type="checkbox" id="ie-crop-lock"> Lock aspect (crop)</label>
<button type="button" class="button" id="ie-rcc">&#8635; Rotate counter-clockwise</button>
<button type="button" class="button" id="ie-rc">&#8634; Rotate clockwise</button>
<button type="button" class="button" id="ie-fv">Flip vertical</button>
<button type="button" class="button" id="ie-fh">Flip horizontal</button>
<span style="border-left:1px solid #c3c4c7;padding-left:8px">Scale:
<input type="number" id="ie-sw" style="width:70px"> × <input type="number" id="ie-sh" style="width:70px">
<label style="font-size:12px"><input type="checkbox" id="ie-aspect" checked> Lock aspect</label>
<button type="button" class="button" id="ie-scale">Scale</button></span>
<span id="ie-msg" style="color:#b32d2e"></span>
</div>
<div style="margin-bottom:8px;color:#646970;font-size:12px" id="ie-info"></div>
<canvas id="ie-cv" style="max-width:100%;border:1px solid #c3c4c7;cursor:crosshair"></canvas>
<div style="margin-top:8px"><button type="button" class="button button-primary" id="ie-save" disabled>Save</button>
<button type="button" class="button" id="ie-cancel">Cancel</button>{restore_btn}</div>
</div>
<script>
(function(){{
var url='{url}',csrf='{csrf}',mid={id};
var img=new Image();img.crossOrigin='anonymous';
var cv=document.getElementById('ie-cv'),ctx=cv.getContext('2d');
var st={{angle:0,fv:false,fh:false,sx:0,sy:0,sw:0,sh:0,dw:0,dh:0,crop:null}};
var dirty=false,orig=null;
function tainted(e){{document.getElementById('ie-msg').textContent='Cannot save: image is served from a different origin.';return true;}}
img.onerror=function(){{document.getElementById('ie-open').style.display='none';}};
img.onload=function(){{
  orig={{w:img.naturalWidth,h:img.naturalHeight}};
  st.sx=0;st.sy=0;st.sw=img.naturalWidth;st.sh=img.naturalHeight;
  st.dw=img.naturalWidth;st.dh=img.naturalHeight;
  try{{render();cv.toDataURL();}}catch(e){{tainted(e);document.getElementById('ie-open').style.display='none';}}
}};
img.src=url;
function rotDims(){{return (st.angle%180!==0)?[st.sh,st.sw]:[st.sw,st.sh];}}
function render(){{
  var src=st.crop||{{x:st.sx,y:st.sy,w:st.sw,h:st.sh}};
  var out=(st.angle%180!==0)?[src.h,src.w]:[src.w,src.h];
  var k=Math.min(st.dw/out[0],st.dh/out[1]);var w=out[0]*k,h=out[1]*k;
  cv.width=w;cv.height=h;
  ctx.save();ctx.translate(w/2,h/2);ctx.rotate(st.angle*Math.PI/180);
  ctx.scale(st.fh?-1:1,st.fv?-1:1);
  var rw=(st.angle%180!==0)?h:w,rh=(st.angle%180!==0)?w:h;
  ctx.drawImage(img,src.x,src.y,src.w,src.h,-rw/2,-rh/2,rw,rh);
  ctx.restore();
  document.getElementById('ie-info').textContent='Crop: '+Math.round(src.x)+','+Math.round(src.y)+' '+Math.round(src.w)+'×'+Math.round(src.h)+' · Output: '+Math.round(w)+'×'+Math.round(h)+'px';
  mark();
}}
function mark(){{dirty=true;document.getElementById('ie-save').disabled=false;}}
document.getElementById('ie-open').addEventListener('click',function(e){{
  e.preventDefault();var p=document.getElementById('ie-panel');
  p.style.display=p.style.display==='none'?'block':'none';
}});
document.getElementById('ie-cancel').addEventListener('click',function(){{document.getElementById('ie-panel').style.display='none';}});
['rcc','rc','fv','fh'].forEach(function(b){{
  document.getElementById('ie-'+b).addEventListener('click',function(){{
    if(b==='rcc')st.angle=(st.angle-90+360)%360;
    if(b==='rc')st.angle=(st.angle+90)%360;
    if(b==='fv')st.fv=!st.fv;
    if(b==='fh')st.fh=!st.fh;
    // Resync output box to new aspect so rotate/flip never downscales.
    var o=(st.angle%180!==0)?[st.sh,st.sw]:[st.sw,st.sh];st.dw=o[0];st.dh=o[1];
    render();
  }});
}});
// Scale inputs keep the current aspect ratio while "Lock aspect" is on —
// typing W fills H and vice versa (WP's ie.imgedit_scale behavior).
function outRatio(){{
  var src=st.crop||{{x:st.sx,y:st.sy,w:st.sw,h:st.sh}};
  var o=(st.angle%180!==0)?[src.h,src.w]:[src.w,src.h];
  return o[0]/o[1];
}}
document.getElementById('ie-sw').addEventListener('input',function(){{
  if(!document.getElementById('ie-aspect').checked)return;
  var w=+this.value;if(w>0)document.getElementById('ie-sh').value=Math.round(w/outRatio());
}});
document.getElementById('ie-sh').addEventListener('input',function(){{
  if(!document.getElementById('ie-aspect').checked)return;
  var h=+this.value;if(h>0)document.getElementById('ie-sw').value=Math.round(h*outRatio());
}});
document.getElementById('ie-scale').addEventListener('click',function(){{
  var w=+document.getElementById('ie-sw').value,h=+document.getElementById('ie-sh').value;
  if(w>0&&h>0){{st.dw=w;st.dh=h;render();}}
}});
// drag-crop on canvas → map display rect back through rotation into source space
var drag=null;
cv.addEventListener('mousedown',function(e){{
  var r=cv.getBoundingClientRect();
  drag={{x:e.clientX-r.left,y:e.clientY-r.top,w:0,h:0}};
}});
window.addEventListener('mousemove',function(e){{
  if(!drag)return;
  var r=cv.getBoundingClientRect();
  drag.w=e.clientX-r.left-drag.x;drag.h=e.clientY-r.top-drag.y;
  // Crop aspect lock: constrain the drag rect to the canvas aspect.
  if(document.getElementById('ie-crop-lock').checked&&cv.width&&cv.height){{
    var ar=cv.width/cv.height;
    if(Math.abs(drag.w)>Math.abs(drag.h)*ar)drag.h=Math.sign(drag.h||1)*Math.abs(drag.w)/ar;
    else drag.w=Math.sign(drag.w||1)*Math.abs(drag.h)*ar;
  }}
  render();
  ctx.strokeStyle='#2271b1';ctx.setLineDash([4,4]);
  ctx.strokeRect(drag.x,drag.y,drag.w,drag.h);
}});
window.addEventListener('mouseup',function(){{
  if(!drag)return;
  var dx=Math.min(drag.x,drag.x+drag.w),dy=Math.min(drag.y,drag.y+drag.h);
  var dw=Math.abs(drag.w),dh=Math.abs(drag.h);drag=null;
  if(dw<4||dh<4){{render();return;}}
  // canvas px → output px → rotate back → add to current crop origin
  var kx=cv.width,ky=cv.height;
  var src=st.crop||{{x:st.sx,y:st.sy,w:st.sw,h:st.sh}};
  var out=(st.angle%180!==0)?[src.h,src.w]:[src.w,src.h];
  var sx=out[0]/kx,sy=out[1]/ky;
  var rx=dx*sx,ry=dy*sy,rw=dw*sx,rh=dh*sy;
  // rotate point back by -angle around output centre
  var a=-st.angle*Math.PI/180,cx=out[0]/2,cy=out[1]/2;
  function unrot(x,y){{var px=x-cx,py=y-cy;return [cx+px*Math.cos(a)-py*Math.sin(a),cy+px*Math.sin(a)+py*Math.cos(a)];}}
  var p1=unrot(rx,ry),p2=unrot(rx+rw,ry+rh);
  var nx=Math.min(p1[0],p2[0]),ny=Math.min(p1[1],p2[1]);
  var nw=Math.abs(p2[0]-p1[0]),nh=Math.abs(p2[1]-p1[1]);
  st.crop={{x:src.x+nx,y:src.y+ny,w:nw,h:nh}};
  render();
}});
document.getElementById('ie-crop').addEventListener('click',function(){{
  if(st.crop){{st.sx=st.crop.x;st.sy=st.crop.y;st.sw=st.crop.w;st.sh=st.crop.h;st.crop=null;st.dw=st.sw;st.dh=st.sh;}}
  render();
}});
document.getElementById('ie-save').addEventListener('click',function(){{
  var mime=url.toLowerCase().endsWith('.png')?'image/png':'image/jpeg';
  cv.toBlob(function(b){{
    if(!b){{document.getElementById('ie-msg').textContent='Export failed.';return;}}
    var fd=new FormData();fd.append('csrf',csrf);fd.append('file',b,'edit.bin');
    fetch('/admin/media/replace/'+mid,{{method:'POST',body:fd}}).then(function(r){{
      document.getElementById('ie-msg').style.color=r.ok?'#00a32a':'#b32d2e';
      document.getElementById('ie-msg').textContent=r.ok?'Saved.':'Save failed.';
      if(r.ok){{dirty=false;document.getElementById('ie-save').disabled=true;
        var bust=url+'?v='+Date.now();
        document.querySelector('.wp_attachment_image img').src=bust;
        img.src=bust;st.angle=0;st.fv=false;st.fh=false;st.crop=null;}}
    }});
  }},mime,0.92);
}});
// Restore Original: POST restore=1 to the same replace endpoint, then reload
// so the preview + "Uploaded on" meta reflect the pristine bytes.
var rb=document.getElementById('ie-restore');
if(rb)rb.addEventListener('click',function(){{
  if(!confirm('Discard all edits and restore the original image?'))return;
  var fd=new FormData();fd.append('csrf',csrf);fd.append('restore','1');
  fetch('/admin/media/replace/'+mid,{{method:'POST',body:fd}}).then(function(r){{
    if(r.ok)location.href='/admin/media/edit/'+mid+'?restored=1';
    else{{var m=document.getElementById('ie-msg');m.style.color='#b32d2e';m.textContent='Restore failed.';}}
  }});
}});
}})();
</script>"##,
        id = id,
        url = url,
        csrf = csrf,
        restore_btn = restore_btn,
    )
}

/// `POST /admin/media/regenerate` — WP "Regenerate Thumbnails" backfill
/// (6.4): rebuild sizes_json + size files for every local image attachment
/// using the current Settings→Media values, then return to the library.
pub(crate) async fn media_regenerate(
    State(app): State<App>,
    headers: HeaderMap,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = check_csrf(&s, f.get("csrf").map(|v| v.as_str()).unwrap_or("")) {
        return r;
    }
    let uploads = sc.www.join("wp-content/uploads");
    match crud::media_regenerate_sizes(&sc.db, &uploads) {
        Ok((done, skipped)) => Redirect::to(&format!(
            "/admin/media?updated=1&regen={done}&skipped={skipped}"
        ))
        .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}")).into_response(),
    }
}

