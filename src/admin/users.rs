//! WP-core-parity Users screens: users.php / user-new.php / user-edit.php /
//! profile.php. Role-gated to `s.role == "admin"` except `/admin/profile`
//! (any session) and self-scoped saves inside `users_save`.
//! Cuts vs WP (recorded in phase spec): gravatar/avatar, Language dropdown
//! (no i18n infra), Send User Notification (no SMTP — rendered disabled),
//! Biographical Info, content attribution on delete (posts has no owner
//! column — Posts count uses post_meta.author_id instead).
use super::*;
use crate::db::users as udb;

/// Every `/admin/users*` mutation/read is admin-only; `/admin/profile` and
/// self-saves inside `users_save` are the two exemptions.
fn require_admin(s: &Sess) -> Result<(), StatusCode> {
    if s.role == "admin" {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

/// `admin-schemes.min.css` styles `body.admin-color-{scheme}` — resolve the
/// session user's persisted scheme (tampered values fall back to modern).
pub(crate) fn color_class(s: &Sess) -> String {
    let p = udb::user_prefs(&s.db, s.user_id);
    if udb::SCHEMES.iter().any(|(k, _)| *k == p.color_scheme) {
        format!("admin-color-{}", p.color_scheme)
    } else {
        "admin-color-modern".into()
    }
}

/// Drop every session owned by `user_id`, keeping `keep_token` alive.
/// `Sess` doesn't store its own token, so callers pass the `rpress_session`
/// cookie value. When a keep-token exists its site scopes the drop
/// (multi-site shares one map, user ids are per-site); `None` drops
/// that id everywhere (used by delete — over-dropping only forces re-login).
/// Integrator: sessions live in `app.sessions: Mutex<HashMap<token, Sess>>`.
pub(crate) fn drop_sessions_for(app: &App, user_id: i64, keep_token: Option<&str>) {
    let mut map = app.sessions.lock();
    let scope: Option<String> = keep_token.and_then(|t| map.get(t).map(|s| s.site.clone()));
    map.retain(|tok, s| {
        if s.user_id != user_id || Some(tok.as_str()) == keep_token {
            return true;
        }
        match &scope {
            Some(site) => s.site != *site,
            None => false,
        }
    });
}

fn role_label(role: &str) -> &'static str {
    match role {
        "admin" => "Administrator",
        "editor" => "Editor",
        _ => "—",
    }
}

fn urlenc(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() || "-_.".contains(c) { c.to_string() } else { format!("%{:02X}", c as u32) })
        .collect()
}

/// `?msg=` → WP `.notice` blocks. `n`/`skipped` feed the count texts.
fn notice_html(msg: &str, n: i64, skipped: i64) -> String {
    let mut out = String::new();
    let (class, text) = match msg {
        "created" => ("notice-success", "New user created.".to_string()),
        "updated" => ("notice-success", "User updated.".to_string()),
        "logout" => ("notice-success", "Logged out of other sessions.".to_string()),
        "deleted" => ("notice-success", format!("{n} user{} deleted.", if n == 1 { "" } else { "s" })),
        "roles" => ("notice-success", format!("Changed role for {n} user{}.", if n == 1 { "" } else { "s" })),
        "err-self" => ("notice-error", "You cannot delete or demote your own account.".to_string()),
        "err-last-admin" => ("notice-error", "You cannot delete or demote the last administrator.".to_string()),
        "err-roles" => ("notice-error", "No roles were changed — you cannot change your own role or the last administrator's.".to_string()),
        "err-dup" => ("notice-error", "That email is already registered.".to_string()),
        "err-pass" => ("notice-error", "Passwords do not match, or are shorter than 8 characters.".to_string()),
        "err-required" => ("notice-error", "A valid email and role are required.".to_string()),
        "err-nouser" => ("notice-error", "User not found.".to_string()),
        "err-appname" => ("notice-error", "Application password name is required.".to_string()),
        "err-appdup" => ("notice-error", "An application password with that name already exists.".to_string()),
        "apprevoked" => ("notice-success", "Application password revoked.".to_string()),
        "sessrevoked" => ("notice-success", "Session logged out.".to_string()),
        _ => ("", String::new()),
    };
    if !text.is_empty() {
        out.push_str(&format!("<div class=\"notice {class} is-dismissible\"><p>{}</p></div>", esc(&text)));
    }
    if skipped > 0 {
        out.push_str(&format!(
            "<div class=\"notice notice-warning is-dismissible\"><p>{skipped} user{} skipped (own account or last administrator).</p></div>",
            if skipped == 1 { "" } else { "s" }
        ));
    }
    out
}

fn parse_ids(csv: &str) -> Vec<i64> {
    csv.split(',')
        .filter_map(|x| x.trim().parse::<i64>().ok())
        .collect()
}

/// Selected ids from a bulk form: `u-{id}` checkboxes.
fn selected_ids(map: &HashMap<String, String>) -> Vec<i64> {
    map.keys()
        .filter_map(|k| k.strip_prefix("u-").and_then(|x| x.parse::<i64>().ok()))
        .collect()
}

/// Ids posted to the delete commit: `ids` csv + `del-{id}` fields.
fn delete_ids(map: &HashMap<String, String>) -> Vec<i64> {
    let mut ids: Vec<i64> = map
        .get("ids")
        .map(|s| parse_ids(s))
        .unwrap_or_default();
    for k in map.keys() {
        if let Some(id) = k.strip_prefix("del-").and_then(|x| x.parse::<i64>().ok()) {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    ids
}

fn back_target(map: &HashMap<String, String>, default: &str) -> String {
    map.get("back")
        .filter(|b| b.starts_with('/') && !b.starts_with("//"))
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

fn redirect_with(url: &str, extra: &str) -> Response {
    let sep = if url.contains('?') { "&" } else { "?" };
    Redirect::to(&format!("{url}{sep}{extra}")).into_response()
}

/// Server-side 16-char generator (same alphabet as the JS one) — used when
/// the new-user password field is left blank.
fn gen_password() -> String {
    const C: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789!@#$%^&*";
    let mut b = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut b);
    b.iter().map(|x| C[(*x as usize) % C.len()] as char).collect()
}

/// Show/Generate/strength JS for the password row (user-new + user-edit).
const PW_JS: &str = r#"<script>
(function(){
var C='ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789!@#$%^&*';
function mk(){var b=new Uint8Array(16);crypto.getRandomValues(b);var o='';for(var i=0;i<16;i++)o+=C[b[i]%C.length];return o}
function upd(){var i=document.getElementById('pass1'),s=document.getElementById('pass-strength');if(!i||!s)return;var v=i.value;var n=(v.length>=12?1:0)+((/[A-Z]/.test(v)&&/[a-z]/.test(v))?1:0)+(/\d/.test(v)?1:0)+(/[^A-Za-z0-9]/.test(v)?1:0);s.textContent=v?('Strength: '+(n>=4?'strong':(n>=2?'medium':'weak'))):''}
var g=document.getElementById('pass-gen');if(g)g.addEventListener('click',function(){var p=mk();var i=document.getElementById('pass1');i.value=p;i.type='text';var j=document.getElementById('pass2');if(j){j.value=p;j.type='text'}upd()});
var h=document.getElementById('pass-show');if(h)h.addEventListener('click',function(){var i=document.getElementById('pass1');var t=i.type==='password';i.type=t?'text':'password';var j=document.getElementById('pass2');if(j)j.type=i.type;this.textContent=t?'Hide':'Show'});
var i=document.getElementById('pass1');if(i)i.addEventListener('input',upd);
})();
</script>"#;

fn role_select(name: &str, current: &str, locked: bool, lock_note: &str) -> String {
    if locked {
        return format!(
            "{} <span class=\"description\">{}</span><input type=\"hidden\" name=\"{name}\" value=\"{}\">",
            esc(role_label(current)),
            esc(lock_note),
            esc(current)
        );
    }
    let sel = |v: &str| if v == current { " selected" } else { "" };
    format!(
        "<select name=\"{name}\" id=\"{name}\"><option value=\"editor\"{}>Editor</option><option value=\"admin\"{}>Administrator</option></select>",
        sel("editor"),
        sel("admin")
    )
}

/// WP "Display name publicly as" variants for `u` (nickname/first+last forms).
fn display_variants(u: &udb::UserRow) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    let mut push = |x: String| {
        let x = x.trim().to_string();
        if !x.is_empty() && !v.contains(&x) {
            v.push(x);
        }
    };
    push(u.display_name.clone());
    push(format!("{} {}", u.first_name, u.last_name));
    push(format!("{} {}", u.last_name, u.first_name));
    push(u.first_name.clone());
    push(u.last_name.clone());
    v
}

/// `users.php` — views All|Administrator|Editor, cb|Username|Name|Email|Role,
/// bulk Delete + separate "Change role to…" control, ?s= search, ?paged=.
pub(crate) async fn users_page(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = require_admin(&s) {
        return r.into_response();
    }
    let role = q.get("role").cloned().unwrap_or_default();
    let role = if role == "admin" || role == "editor" { role } else { String::new() };
    let search = q.get("s").cloned().unwrap_or_default();

    // Screen Options (GET form like media/edit-tags): `screen_options=1`
    // marks a submit; absent `hide_*` boxes mean "hidden". Persists to
    // `settings.users_screen_{user_id}` as {"hide":[cols],"per_page":n}.
    const UCOLS: [&str; 4] = ["name", "email", "role", "posts"];
    let screen_key = format!("users_screen_{}", s.user_id);
    if q.get("screen_options").is_some() {
        let hide: Vec<String> = UCOLS
            .iter()
            .filter(|c| !q.contains_key(&format!("hide_{c}")))
            .map(|c| c.to_string())
            .collect();
        let per = q.get("per_page").and_then(|v| v.parse::<i64>().ok()).unwrap_or(20).clamp(1, 500);
        let json = serde_json::json!({"hide": hide, "per_page": per}).to_string();
        let _ = crud::comments::setting_set(&sc.db, &screen_key, &json);
        return Redirect::to("/admin/users").into_response();
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

    let counts = udb::counts_by_role(&sc.db).unwrap_or_default();
    let all: i64 = counts.values().sum();
    let total = udb::count(&sc.db, &role, &search).unwrap_or(0);
    let pager = list::Pager::new(&q, total).with_per_page(per_page);
    // Fetch the full filtered page-set (users are few) so the Posts column —
    // a per-row COUNT over post_meta.author_id — can participate in sort.
    let mut rows = udb::list(
        &sc.db,
        &udb::Filter { role: &role, s: &search, limit: total.max(1), offset: 0 },
    )
    .unwrap_or_default();
    let orderby = q.get("orderby").map(|v| v.as_str()).unwrap_or("");
    let desc = q.get("order").map(|v| v.as_str()) == Some("desc");
    let mut post_counts: HashMap<i64, i64> = HashMap::new();
    for u in &rows {
        post_counts.insert(u.id, udb::posts_count(&sc.db, u.id).unwrap_or(0));
    }
    match orderby {
        "posts" => rows.sort_by(|a, b| {
            let c = post_counts[&a.id].cmp(&post_counts[&b.id]);
            let c = if desc { c.reverse() } else { c };
            c.then_with(|| a.email.to_lowercase().cmp(&b.email.to_lowercase()))
        }),
        "username" if desc => rows
            .sort_by(|a, b| b.email.to_lowercase().cmp(&a.email.to_lowercase())),
        _ => {}
    }
    // Manual slice — the DB page query ran unpaged above.
    let rows: Vec<udb::UserRow> = rows
        .into_iter()
        .skip(pager.offset() as usize)
        .take(pager.limit() as usize)
        .collect();

    // Canonical list URL — row Edit/Delete and `back` fields return here.
    let mut lp: Vec<String> = Vec::new();
    if !role.is_empty() {
        lp.push(format!("role={role}"));
    }
    if !search.is_empty() {
        lp.push(format!("s={}", urlenc(&search)));
    }
    if pager.page > 1 {
        lp.push(format!("paged={}", pager.page));
    }
    let list_url = if lp.is_empty() { "/admin/users".to_string() } else { format!("/admin/users?{}", lp.join("&")) };
    let back_enc = urlenc(&list_url);

    let mut view_extra = HashMap::new();
    view_extra.insert("s".to_string(), search.clone());
    let views = [
        list::View { key: "all", label: "All", count: all },
        list::View { key: "admin", label: "Administrator", count: *counts.get("admin").unwrap_or(&0) },
        list::View { key: "editor", label: "Editor", count: *counts.get("editor").unwrap_or(&0) },
    ];
    let current = if role.is_empty() { "all" } else { role.as_str() };
    let subsubsub = list::views_bar("/admin/users?", "role", current, &views, &view_extra);

    let mut nav_extra = HashMap::new();
    nav_extra.insert("role".to_string(), role.clone());
    nav_extra.insert("s".to_string(), search.clone());
    let pager_nav = pager.nav("/admin/users", &nav_extra);

    let mut body = String::new();
    for u in &rows {
        let username = if u.display_name.is_empty() { u.email.clone() } else { u.display_name.clone() };
        let name = format!("{} {}", u.first_name, u.last_name);
        let name = name.trim();
        // WP never offers Delete on the logged-in user's own row.
        let delete_span = if u.id == s.user_id {
            String::new()
        } else {
            format!(
                " | <span class=\"delete\"><a href=\"/admin/users/delete?ids={}&amp;back={}\">Delete</a></span>",
                u.id, back_enc
            )
        };
        body.push_str(&format!(
            r#"<tr id="user-{id}"><th scope="row" class="check-column"><input type="checkbox" name="u-{id}"></th>
<td class="username column-username has-row-actions column-primary"><strong><a class="row-title" href="/admin/user-edit/{id}?back={back_enc}">{username}</a></strong>
<div class="row-actions"><span class="edit"><a href="/admin/user-edit/{id}?back={back_enc}">Edit</a></span>{delete_span}</div></td>
<td class="name column-name{h_name}">{name}</td>
<td class="email column-email{h_email}"><a href="mailto:{email}">{email}</a></td>
<td class="role column-role{h_role}">{rlabel}</td>
<td class="posts column-posts{h_posts}">{posts}</td></tr>"#,
            id = u.id,
            back_enc = back_enc,
            username = esc(&username),
            delete_span = delete_span,
            name = if name.is_empty() { "—".into() } else { esc(name) },
            email = esc(&u.email),
            rlabel = role_label(&u.role),
            h_name = col_cls("name"),
            h_email = col_cls("email"),
            h_role = col_cls("role"),
            h_posts = col_cls("posts"),
            posts = post_counts.get(&u.id).copied().unwrap_or(0),
        ));
    }
    if body.is_empty() {
        body.push_str(&list::empty_row(6, "No users found."));
    }

    // WP renders the bulk + change-role controls on both tablenavs. The
    // BOTTOM pair uses `action2`/`new_role2` — serde_urlencoded last-wins
    // would let an untouched bottom select shadow a chosen top action.
    let role_actions = |id_suffix: &str, name: &str| {
        format!(
            r#"<div class="alignleft actions"><label class="screen-reader-text" for="{name}{id_suffix}">Change role to…</label>
<select name="{name}" id="{name}{id_suffix}"><option value="">Change role to…</option><option value="editor">Editor</option><option value="admin">Administrator</option></select>
<input type="submit" name="changeit" id="changeit{id_suffix}" class="button" value="Change"></div>"#
        )
    };
    let bulk_actions = |sel_id: &str, name: &str| {
        format!(
            r#"<div class="alignleft actions bulkactions"><label class="screen-reader-text" for="{sel_id}">Select bulk action</label>
<select name="{name}" id="{sel_id}"><option value="-1">Bulk actions</option><option value="delete">Delete</option></select>
<input type="submit" class="button action" value="Apply"></div>"#
        )
    };
    let notice = notice_html(
        q.get("msg").map(|s| s.as_str()).unwrap_or(""),
        q.get("n").and_then(|s| s.parse().ok()).unwrap_or(0),
        q.get("skipped").and_then(|s| s.parse().ok()).unwrap_or(0),
    );
    let role_hidden = if role.is_empty() { String::new() } else { format!("<input type=\"hidden\" name=\"role\" value=\"{role}\">") };
    let mut extra_q = HashMap::new();
    if !role.is_empty() {
        extra_q.insert("role".into(), role.clone());
    }
    if !search.is_empty() {
        extra_q.insert("s".into(), search.clone());
    }
    let sort_posts = format!("/admin/users?orderby=posts&order={}{}", if desc { "asc" } else { "desc" }, list::qs_pub(&extra_q));
    let sort_name = format!("/admin/users?orderby=username&order={}{}", if desc { "asc" } else { "desc" }, list::qs_pub(&extra_q));
    let h = |n: &str| col_cls(n);
    let head = list::thead(&[
        ("username column-primary", "Username", &sort_name),
        (&format!("name{}", h("name")), "Name", ""),
        (&format!("email{}", h("email")), "Email", ""),
        (&format!("role{}", h("role")), "Role", ""),
        (&format!("posts num{}", h("posts")), "Posts", &sort_posts),
    ]);
    let screen_meta = users_screen_meta(&hide_cols, per_page, &s.csrf, &role, &search);
    let inner = format!(
        r#"{chrome}{screen_meta}<style>
#screen-meta-links{{float:right;margin:0 20px 0 0}}
#screen-meta-links .show-settings{{border:1px solid #c3c4c7;border-top:0;background:#f6f7f7;padding:3px 12px;border-radius:0 0 4px 4px}}
#screen-options-wrap{{background:#fff;border:1px solid #c3c4c7;padding:15px 20px;box-shadow:0 2px 5px rgba(0,0,0,.15)}}
#screen-options-wrap.hidden,.wp-list-table .hidden{{display:none}}
</style><div class="wrap">
<h1 class="wp-heading-inline">Users</h1>
<a href="/admin/user-new" class="page-title-action">Add New User</a>
<hr class="wp-header-end">
{notice}
{subsubsub}
<form method="get" action="/admin/users">
{role_hidden}
<p class="search-box"><label class="screen-reader-text" for="user-search-input">Search Users:</label>
<input type="search" id="user-search-input" name="s" value="{search_val}">
<input type="submit" id="search-submit" class="button" value="Search Users"></p>
</form>
<form method="post" action="/admin/users/bulk">
<input type="hidden" name="csrf" value="{csrf}">
<input type="hidden" name="back" value="{list_url}">
<div class="tablenav top">
{bulk_top}
{role_top}
{pager_nav}
<br class="clear">
</div>
<table class="wp-list-table widefat fixed striped table-view-list users">
{head}
<tbody id="the-list">{body}</tbody>
</table>
<div class="tablenav bottom">
{bulk_bottom}
{role_bottom}
<br class="clear">
</div>
</form>
</div>
<script>
(function(){{
var btn=document.getElementById('show-settings-link');
if(btn)btn.addEventListener('click',function(){{
  var w=document.getElementById('screen-options-wrap');
  var hid=w.classList.toggle('hidden');
  btn.setAttribute('aria-expanded',hid?'false':'true');
}});
// Live column toggle; persistence rides the screen_options GET submit.
document.addEventListener('change',function(e){{
  var t=e.target;
  if(!t||!t.classList||!t.classList.contains('hide-column-tog'))return;
  var cells=document.querySelectorAll('.column-'+t.value);
  for(var i=0;i<cells.length;i++)cells[i].classList.toggle('hidden',!t.checked);
}});
}})();
</script>"#,
        chrome = chrome("users", &s),
        screen_meta = screen_meta,
        notice = notice,
        subsubsub = subsubsub,
        role_hidden = role_hidden,
        search_val = esc(&search),
        csrf = s.csrf,
        list_url = esc(&list_url),
        bulk_top = bulk_actions("bulk-action-selector-top", "action"),
        role_top = role_actions("", "new_role"),
        pager_nav = pager_nav,
        head = head,
        body = body,
        bulk_bottom = bulk_actions("bulk-action-selector-bottom", "action2"),
        role_bottom = role_actions("2", "new_role2"),
    );
    html_admin(&s, "Users", &inner, true, "users-php").into_response()
}
/// WP `#screen-meta` + `#screen-meta-links` for `users.php`: Screen Options
/// tab — `hide_*` column checkboxes + items-per-page, submitted as a GET
/// back into `users_page` (`screen_options=1` persists to settings).
fn users_screen_meta(hide: &[String], per_page: i64, csrf: &str, role: &str, search: &str) -> String {
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
<form id="adv-settings" method="get" action="/admin/users">
<input type="hidden" name="screen_options" value="1">
<input type="hidden" name="csrf" value="{csrf}">
<input type="hidden" name="role" value="{role}">
<input type="hidden" name="s" value="{search}">
<fieldset class="metabox-prefs view-options">
<legend>Columns</legend>
{cb_name}
{cb_email}
{cb_role}
{cb_posts}
</fieldset>
<fieldset class="screen-options"><legend>Pagination</legend>
<label for="users_per_page">Number of items per page:</label>
<input type="number" step="1" min="1" max="500" class="screen-per-page" name="per_page" id="users_per_page" value="{per_page}">
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
        role = esc(role),
        search = esc(search),
        cb_name = cb("name", "Name"),
        cb_email = cb("email", "Email"),
        cb_role = cb("role", "Role"),
        cb_posts = cb("posts", "Posts"),
        per_page = per_page,
    )
}


/// `user-new.php` — Email (required, doubles as the login/username), names,
/// nickname, website, password (Show + Generate + strength hint), Role.
/// CUT vs WP: Language dropdown (no i18n infra) and Send User Notification
/// checkbox (no SMTP mailer) — recorded in phase spec.
pub(crate) async fn user_new_page(State(app): State<App>, headers: HeaderMap, Query(q): Query<HashMap<String, String>>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    let _ = sc;
    if let Err(r) = require_admin(&s) {
        return r.into_response();
    }
    let notice = notice_html(q.get("msg").map(|s| s.as_str()).unwrap_or(""), 0, 0);
    let inner = format!(
        r#"{chrome}<div class="wrap">
<h1>Add New User</h1>
{notice}
<p>Create a brand new user and add them to this site.</p>
<form method="post" action="/admin/users/create" id="createuser" class="validate" novalidate="novalidate">
<input type="hidden" name="csrf" value="{csrf}">
<table class="form-table" role="presentation">
<tr class="form-field form-required">
<th scope="row"><label for="email">Email <span class="description">(required)</span></label></th>
<td><input name="email" type="email" id="email" value="" aria-required="true" required><p class="description">The email is the login id — it doubles as the WP username and cannot be changed later here.</p></td>
</tr>
<tr class="form-field">
<th scope="row"><label for="first_name">First Name</label></th>
<td><input name="first_name" type="text" id="first_name" value=""></td>
</tr>
<tr class="form-field">
<th scope="row"><label for="last_name">Last Name</label></th>
<td><input name="last_name" type="text" id="last_name" value=""></td>
</tr>
<tr class="form-field">
<th scope="row"><label for="nickname">Nickname</label></th>
<td><input name="nickname" type="text" id="nickname" value=""><p class="description">Shown publicly if no other display name is picked.</p></td>
</tr>
<tr class="form-field">
<th scope="row"><label for="url">Website</label></th>
<td><input name="url" type="url" id="url" value=""></td>
</tr>
<tr class="form-field form-required user-pass1-wrap">
<th scope="row"><label for="pass1">Password</label></th>
<td><input type="password" name="pass1" id="pass1" autocomplete="new-password" spellcheck="false">
<button type="button" class="button" id="pass-show">Show</button>
<button type="button" class="button button-secondary" id="pass-gen">Generate</button>
<p class="description" id="pass-strength"></p>
<p class="description">Leave blank to auto-generate — the password is shown once on the next screen.</p></td>
</tr>
<tr class="form-field form-required">
<th scope="row"><label for="role">Role</label></th>
<td><select name="role" id="role"><option value="editor">Editor</option><option value="admin">Administrator</option></select></td>
</tr>
<tr class="form-field user-send-notification-wrap">
<th scope="row">Send User Notification</th>
<td><label for="send_user_notification"><input name="send_user_notification" type="checkbox" id="send_user_notification" value="1" disabled> Send the new user an email about their account.</label><p class="description">Requires SMTP — not configured in this build.</p></td>
</tr>
</table>
<p class="submit"><input type="submit" name="createuser" id="createusersub" class="button button-primary" value="Add New User"></p>
</form>
{pw_js}
</div>"#,
        chrome = chrome("users", &s),
        notice = notice,
        csrf = s.csrf,
        pw_js = PW_JS,
    );
    html_admin(&s, "Add New User", &inner, true, "user-new-php").into_response()
}

#[derive(Deserialize)]
pub(crate) struct CreateForm {
    csrf: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    first_name: String,
    #[serde(default)]
    last_name: String,
    #[serde(default)]
    nickname: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    pass1: String,
    #[serde(default)]
    role: String,
}

pub(crate) async fn users_create(State(app): State<App>, headers: HeaderMap, Form(f): Form<CreateForm>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = require_admin(&s) {
        return r.into_response();
    }
    if let Err(r) = check_csrf(&s, &f.csrf) {
        return r;
    }
    let email = f.email.trim().to_lowercase();
    let role = f.role.trim().to_string();
    if !email.contains('@') || !matches!(role.as_str(), "admin" | "editor") {
        return Redirect::to("/admin/user-new?msg=err-required").into_response();
    }
    // Blank password → server-generated; shown once on a result screen
    // (WP mails it — no SMTP here, recorded as the pragmatic equivalent).
    let generated = if f.pass1.is_empty() { Some(gen_password()) } else { None };
    let pw = generated.clone().unwrap_or_else(|| f.pass1.clone());
    if pw.len() < 8 {
        return Redirect::to("/admin/user-new?msg=err-pass").into_response();
    }
    let Ok(hash) = crate_auth::hash_password(&pw) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    match udb::create(
        &sc.db,
        &email,
        &hash,
        &role,
        f.nickname.trim(),
        f.first_name.trim(),
        f.last_name.trim(),
        f.url.trim(),
    ) {
        Ok(_) if generated.is_none() => Redirect::to("/admin/users?msg=created").into_response(),
        Ok(_) => {
            let pw_shown = esc(generated.as_deref().unwrap_or(""));
            let inner = format!(
                r#"{chrome}<div class="wrap"><h1>Users</h1>
<div class="notice notice-success"><p>New user created.</p></div>
<div class="notice notice-warning"><p>Generated password — shown once only: <code>{pw_shown}</code></p></div>
<p><a class="button" href="/admin/users">All Users</a></p></div>"#,
                chrome = chrome("users", &s),
                pw_shown = pw_shown,
            );
            html_admin(&s, "Users", &inner, true, "users-php").into_response()
        }
        Err(_) => Redirect::to("/admin/user-new?msg=err-dup").into_response(),
    }
}

/// Shared edit/profile form. `profile` drops the Role row entirely (WP
/// profile.php parity — not read-only, omitted).
fn user_edit_html(s: &Sess, u: &udb::UserRow, profile: bool, last_admin: bool, back: &str, notice: &str) -> String {
    let self_edit = u.id == s.user_id;
    let role_locked = self_edit || last_admin;
    let lock_note = if self_edit {
        "You cannot change your own role."
    } else {
        "The last administrator's role is locked."
    };
    let role_row = if profile {
        String::new()
    } else {
        format!(
            "<tr class=\"user-role-wrap\"><th scope=\"row\"><label for=\"role\">Role</label></th><td>{}</td></tr>",
            role_select("role", &u.role, role_locked, lock_note)
        )
    };
    let prefs = udb::user_prefs(&s.db, u.id);
    let mut scheme_radios = String::new();
    for (key, label) in udb::SCHEMES {
        let chk = if prefs.color_scheme == *key { " checked" } else { "" };
        scheme_radios.push_str(&format!("<label><input type=\"radio\" name=\"color_scheme\" value=\"{}\"{}> {}</label><br>", esc(key), chk, esc(label)));
    }
    let rich_checked = if prefs.rich_editing { " checked" } else { "" };
    let toolbar_checked = if prefs.show_toolbar { " checked" } else { "" };
    let variants = display_variants(u);
    let mut opts = String::new();
    for v in &variants {
        let sel = if *v == u.display_name { " selected" } else { "" };
        opts.push_str(&format!("<option value=\"{}\"{}>{}</option>", esc(v), sel, esc(v)));
    }
    // Sessions row: button posts to a form declared outside this <form>
    // (nested forms are dropped by parsers — `form=` attribute pattern).
    let sessions = udb::session_list(&s.db, u.id).unwrap_or_default();
    let mut sessions_tbl = String::new();
    for ss in sessions {
        sessions_tbl.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>", esc(&ss.ua), esc(&ss.ip), esc(&ss.created_at), esc(&ss.last_seen)));
    }
    if sessions_tbl.is_empty() {
        sessions_tbl.push_str("<tr><td colspan=\"4\">No persisted sessions found.</td></tr>");
    }
    let sessions_row = format!(
        r#"<tr class="user-sessions-wrap"><th scope="row">Sessions</th><td>
<table class="widefat striped"><thead><tr><th>User Agent</th><th>IP</th><th>Created</th><th>Last seen</th></tr></thead><tbody>{sessions_tbl}</tbody></table>
<p><button type="submit" form="logout-others-form" class="button">Log Out Other Sessions</button></p>
<p class="description">Left somewhere logged in? You can log out everywhere else, and keep this session.</p></td></tr>"#
    );
    let passes = udb::app_pass_list(&s.db, u.id).unwrap_or_default();
    let mut pass_rows = String::new();
    for p in passes {
        pass_rows.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><form method=\"post\" action=\"/admin/users/app-pass/revoke\"><input type=\"hidden\" name=\"csrf\" value=\"{}\"><input type=\"hidden\" name=\"user_id\" value=\"{}\"><input type=\"hidden\" name=\"id\" value=\"{}\"><button class=\"button-link delete\">Revoke</button></form></td></tr>", esc(&p.name), esc(&p.created_at), esc(&p.last_used), esc(&p.last_ip), esc(&s.csrf), u.id, p.id));
    }
    if pass_rows.is_empty() { pass_rows.push_str("<tr><td colspan=\"5\">No application passwords.</td></tr>"); }
    let app_passwords = format!(r#"<h2>Application Passwords</h2>
<table class="form-table"><tr><th scope="row">New Application Password Name</th><td>
<form method="post" action="/admin/users/app-pass/add"><input type="hidden" name="csrf" value="{csrf}"><input type="hidden" name="user_id" value="{uid}"><input name="name" class="regular-text" placeholder="e.g. API client"> <button class="button">Add New Application Password</button></form>
<p class="description">Use these for Basic Auth on `/api/v1/*`; the password is shown once.</p>
<table class="widefat striped"><thead><tr><th>Name</th><th>Created</th><th>Last used</th><th>Last IP</th><th>Action</th></tr></thead><tbody>{pass_rows}</tbody></table>
</td></tr></table>"#, csrf=esc(&s.csrf), uid=u.id, pass_rows=pass_rows);
    let nick_val = if u.display_name.is_empty() { "" } else { u.display_name.as_str() };
    let h1 = if profile {
        "Profile".to_string()
    } else {
        format!("Edit User {}", if u.display_name.is_empty() { &u.email } else { &u.display_name })
    };
    format!(
        r#"{chrome}<div class="wrap">
<h1>{h1}</h1>
{notice}
<form method="post" action="/admin/users/save" id="your-profile" class="validate" novalidate="novalidate">
<input type="hidden" name="csrf" value="{csrf}">
<input type="hidden" name="id" value="{id}">
<input type="hidden" name="back" value="{back}">
<h2>Personal Options</h2>
<table class="form-table" role="presentation">
<tr class="user-rich-editing-wrap"><th scope="row">Visual Editor</th><td><label><input type="checkbox" name="rich_editing" value="1"{rich_checked}> Enable the visual editor when writing</label></td></tr>
<tr class="user-admin-color-wrap"><th scope="row">Admin Color Scheme</th><td>{scheme_radios}</td></tr>
<tr class="show-admin-bar user-admin-bar-front-wrap"><th scope="row">Toolbar</th><td><label><input type="checkbox" name="show_toolbar" value="1"{toolbar_checked}> Show Toolbar when viewing site</label></td></tr>
<tr class="user-user-login-wrap">
<th scope="row"><label for="user_login">Username</label></th>
<td><input type="text" id="user_login" value="{email}" disabled="disabled" class="regular-text"> <span class="description">Usernames cannot be changed — the email is the login id.</span></td>
</tr>
{role_row}
</table>
<h2>Name</h2>
<table class="form-table" role="presentation">
<tr class="user-first-name-wrap">
<th scope="row"><label for="first_name">First Name</label></th>
<td><input type="text" name="first_name" id="first_name" value="{first_name}" class="regular-text"></td>
</tr>
<tr class="user-last-name-wrap">
<th scope="row"><label for="last_name">Last Name</label></th>
<td><input type="text" name="last_name" id="last_name" value="{last_name}" class="regular-text"></td>
</tr>
<tr class="user-nickname-wrap">
<th scope="row"><label for="nickname">Nickname <span class="description">(required)</span></label></th>
<td><input type="text" name="nickname" id="nickname" value="{nick}" class="regular-text" required></td>
</tr>
<tr class="user-display-name-wrap">
<th scope="row"><label for="display_name">Display name publicly as</label></th>
<td><select name="display_name" id="display_name">{opts}</select></td>
</tr>
</table>
<h2>Contact Info</h2>
<table class="form-table" role="presentation">
<tr class="user-email-wrap">
<th scope="row"><label for="email">Email <span class="description">(required)</span></label></th>
<td><input type="email" name="email" id="email" value="{email}" class="regular-text ltr" aria-required="true" required></td>
</tr>
<tr class="user-url-wrap">
<th scope="row"><label for="url">Website</label></th>
<td><input type="url" name="url" id="url" value="{website}" class="regular-text code"></td>
</tr>
</table>
<h2>Account Management</h2>
<table class="form-table" role="presentation">
{sessions_row}
<tr id="password" class="user-pass1-wrap">
<th scope="row"><label for="pass1">New Password</label></th>
<td><input type="password" name="pass1" id="pass1" class="regular-text" autocomplete="new-password" spellcheck="false">
<button type="button" class="button" id="pass-show">Show</button>
<button type="button" class="button button-secondary" id="pass-gen">Generate</button>
<p class="description" id="pass-strength"></p>
<p class="description">Leave both fields blank to keep the current password. Setting a new one logs this user out everywhere else.</p></td>
</tr>
<tr class="user-pass2-wrap">
<th scope="row"><label for="pass2">Repeat New Password</label></th>
<td><input type="password" name="pass2" id="pass2" class="regular-text" autocomplete="new-password" spellcheck="false"></td>
</tr>
</table>
<p class="submit"><input type="submit" class="button button-primary" value="{submit}"></p>
</form>
{app_passwords}
<form method="post" action="/admin/users/logout-others" id="logout-others-form">
<input type="hidden" name="csrf" value="{csrf}">
<input type="hidden" name="id" value="{id}">
<input type="hidden" name="back" value="{back}">
</form>
{pw_js}
</div>"#,
        chrome = chrome("users", s),
        h1 = esc(&h1),
        notice = notice,
        csrf = s.csrf,
        id = u.id,
        back = esc(back),
        email = esc(&u.email),
        role_row = role_row,
        rich_checked = rich_checked,
        toolbar_checked = toolbar_checked,
        scheme_radios = scheme_radios,
        first_name = esc(&u.first_name),
        last_name = esc(&u.last_name),
        nick = esc(nick_val),
        opts = opts,
        website = esc(&u.website),
        sessions_row = sessions_row,
        app_passwords = app_passwords,
        submit = if profile { "Update Profile" } else { "Update User" },
        pw_js = PW_JS,
    )
}

/// `user-edit.php` — full edit incl. Role (locked for self and last admin)
/// and "Log Out Other Sessions".
pub(crate) async fn user_edit_page(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = require_admin(&s) {
        return r.into_response();
    }
    let Ok(Some(u)) = udb::get(&sc.db, id) else {
        return redirect_with("/admin/users", "msg=err-nouser");
    };
    let back = q
        .get("back")
        .filter(|b| b.starts_with('/') && !b.starts_with("//"))
        .cloned()
        .unwrap_or_else(|| "/admin/users".into());
    // HARDENING (beyond WP): the sole remaining admin's role is locked.
    let last_admin = u.role == "admin" && udb::count_admins(&sc.db).unwrap_or(0) <= 1;
    let notice = notice_html(q.get("msg").map(|s| s.as_str()).unwrap_or(""), 0, 0);
    let inner = user_edit_html(&s, &u, false, last_admin, &back, &notice);
    html_admin(&s, "Edit User", &inner, true, "user-edit-php").into_response()
}

#[derive(Deserialize)]
pub(crate) struct SaveForm {
    csrf: String,
    id: i64,
    #[serde(default)]
    email: String,
    #[serde(default)]
    first_name: String,
    #[serde(default)]
    last_name: String,
    #[serde(default)]
    nickname: String,
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    pass1: String,
    #[serde(default)]
    pass2: String,
    #[serde(default)]
    role: String,
    #[serde(default)]
    color_scheme: String,
    #[serde(default)]
    rich_editing: Option<String>,
    #[serde(default)]
    show_toolbar: Option<String>,
    #[serde(default)]
    back: String,
}

/// Commit for user-edit AND profile. Non-admins may only save their own row
/// (`id == s.user_id`); the role field is ignored for self/last-admin even if
/// a crafted form submits it.
pub(crate) async fn users_save(State(app): State<App>, headers: HeaderMap, Form(f): Form<SaveForm>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    let is_self = f.id == s.user_id;
    if !is_self {
        if let Err(r) = require_admin(&s) {
            return r.into_response();
        }
    }
    if let Err(r) = check_csrf(&s, &f.csrf) {
        return r;
    }
    // Form errors return to the screen being edited (WP update=1 parity);
    // success returns to the preserved list view via `back`.
    let err_back = if f.back == "/admin/profile" && is_self {
        "/admin/profile".to_string()
    } else {
        format!("/admin/user-edit/{}", f.id)
    };
    let back = if f.back.starts_with('/') && !f.back.starts_with("//") { f.back.clone() } else { err_back.clone() };
    let Ok(Some(cur)) = udb::get(&sc.db, f.id) else {
        return redirect_with("/admin/users", "msg=err-nouser");
    };
    // Password validated before any write so a mismatch never half-saves.
    let new_pw: Option<String> = if f.pass1.is_empty() && f.pass2.is_empty() {
        None
    } else if f.pass1 != f.pass2 || f.pass1.len() < 8 {
        return redirect_with(&err_back, "msg=err-pass");
    } else {
        Some(f.pass1.clone())
    };
    let email = f.email.trim().to_lowercase();
    if !email.contains('@') {
        return redirect_with(&err_back, "msg=err-required");
    }
    let display = if f.display_name.trim().is_empty() {
        f.nickname.trim().to_string()
    } else {
        f.display_name.trim().to_string()
    };
    // Role: never trust the field for self or the last admin — keep current.
    let last_admin = cur.role == "admin" && udb::count_admins(&sc.db).unwrap_or(0) <= 1;
    let role = if is_self || last_admin || !matches!(f.role.as_str(), "admin" | "editor") {
        cur.role.clone()
    } else {
        f.role.clone()
    };
    if udb::update(&sc.db, f.id, &email, &display, f.first_name.trim(), f.last_name.trim(), f.url.trim(), &role).is_err() {
        return redirect_with(&err_back, "msg=err-dup");
    }
    let scheme = if udb::SCHEMES.iter().any(|(k, _)| *k == f.color_scheme) { f.color_scheme.clone() } else { "modern".into() };
    let _ = udb::save_user_prefs(&sc.db, f.id, &udb::UserPrefs {
        color_scheme: scheme,
        rich_editing: f.rich_editing.is_some(),
        show_toolbar: f.show_toolbar.is_some(),
    });
    // Role change invalidates the victim's sessions immediately (A9) — a
    // demoted user must not keep acting with the stale in-memory role.
    if !is_self && !last_admin && role != cur.role {
        let _ = udb::session_delete_user(&sc.db, f.id);
        drop_sessions_for(&app, f.id, None);
    }
    if let Some(pw) = new_pw {
        let Ok(hash) = crate_auth::hash_password(&pw) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };
        // Propagate the write failure — a silent success would leave the
        // user believing the password changed when it didn't (A11).
        if udb::update_password(&sc.db, f.id, &hash).is_err() {
            return (StatusCode::INTERNAL_SERVER_ERROR, "password not changed").into_response();
        }
        // Password set → kill that user's other sessions (WP does the same).
        let keep = if is_self { cookie(&headers, SESSION_COOKIE) } else { None };
        let _ = udb::session_delete_others(&sc.db, f.id, keep.as_deref().unwrap_or(""));
        drop_sessions_for(&app, f.id, keep.as_deref());
    }
    redirect_with(&back, "msg=updated")
}

/// "Log Out Other Sessions" — drops every session of `id` except the
/// current one when acting on self. Admin-gated for other users.
#[derive(Deserialize)]
pub(crate) struct LogoutForm {
    csrf: String,
    id: i64,
    #[serde(default)]
    back: String,
}

pub(crate) async fn users_logout_others(State(app): State<App>, headers: HeaderMap, Form(f): Form<LogoutForm>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = check_csrf(&s, &f.csrf) {
        return r;
    }
    if f.id != s.user_id {
        if let Err(r) = require_admin(&s) {
            return r.into_response();
        }
    }
    let keep = if f.id == s.user_id { cookie(&headers, SESSION_COOKIE) } else { None };
    // Persisted sessions die too — without this the other device just
    // rehydrates from `sessions` on its next request.
    let _ = udb::session_delete_others(&sc.db, f.id, keep.as_deref().unwrap_or(""));
    drop_sessions_for(&app, f.id, keep.as_deref());
    // WP stays on the same screen — the list `back` is for save/delete only.
    let back = if f.back == "/admin/profile" && f.id == s.user_id {
        "/admin/profile".to_string()
    } else {
        format!("/admin/user-edit/{}", f.id)
    };
    redirect_with(&back, "msg=logout")
}

/// `users.php?action=delete` — WP's two-step confirm screen listing the
/// selected users plus the content-ownership radio. `posts` has no owner
/// column, so "Attribute all content to:" is rendered disabled with a note;
/// "Delete all content" is the only working option.
pub(crate) async fn users_delete_page(
    State(app): State<App>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = require_admin(&s) {
        return r.into_response();
    }
    let ids = parse_ids(q.get("ids").map(|s| s.as_str()).unwrap_or(""));
    let back = q
        .get("back")
        .filter(|b| b.starts_with('/') && !b.starts_with("//"))
        .cloned()
        .unwrap_or_else(|| "/admin/users".into());
    if ids.is_empty() {
        return Redirect::to(&back).into_response();
    }
    let mut items = String::new();
    for id in &ids {
        if let Ok(Some(u)) = udb::get(&sc.db, *id) {
            let name = if u.display_name.is_empty() { &u.email } else { &u.display_name };
            let self_note = if *id == s.user_id { " <em>(you — deletion of your own account is refused)</em>" } else { "" };
            items.push_str(&format!("<li>ID #{id}: {} — {}{self_note}</li>", esc(name), esc(&u.email)));
        }
    }
    if items.is_empty() {
        return Redirect::to(&back).into_response();
    }
    let inner = format!(
        r#"{chrome}<div class="wrap">
<h1>Delete Users</h1>
<p>You have specified these users for deletion:</p>
<ul>{items}</ul>
<form method="post" action="/admin/users/delete">
<input type="hidden" name="csrf" value="{csrf}">
<input type="hidden" name="ids" value="{ids}">
<input type="hidden" name="back" value="{back}">
<fieldset>
<p><legend>What should be done with content owned by these users?</legend></p>
<ul>
<li><label><input type="radio" id="delete_option0" name="delete_option" value="delete" checked="checked"> Delete all content.</label></li>
<li><label for="delete_option1"><input type="radio" id="delete_option1" name="delete_option" value="reassign" disabled="disabled"> Attribute all content to: </label>
<select name="reassign_user" id="reassign_user" disabled="disabled"><option value="">—</option></select>
<p class="description">Attribution is unavailable — posts have no owner column.</p></li>
</ul></fieldset>
<input type="submit" name="submit" class="button button-primary" value="Confirm Deletion">
</form>
</div>"#,
        chrome = chrome("users", &s),
        items = items,
        csrf = s.csrf,
        ids = esc(&ids.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",")),
        back = esc(&back),
    );
    html_admin(&s, "Delete Users", &inner, true, "users-php").into_response()
}

/// Delete commit — self and last-admin are refused per user (WP self-guard +
/// intentional hardening). Sessions of deleted users are dropped.
pub(crate) async fn users_delete(State(app): State<App>, headers: HeaderMap, Form(map): Form<HashMap<String, String>>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = require_admin(&s) {
        return r.into_response();
    }
    if map.get("csrf").map(|x| x.as_str()) != Some(s.csrf.as_str()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let ids = delete_ids(&map);
    let back = back_target(&map, "/admin/users");
    if ids.is_empty() {
        return Redirect::to(&back).into_response();
    }
    // Track remaining admins inside the loop so selecting every admin in one
    // request still leaves ≥1 (beyond-WP hardening, mirroring the bulk path).
    let mut admins_left = udb::count_admins(&sc.db).unwrap_or(0);
    let (mut deleted, mut skipped, mut err) = (0i64, 0i64, String::new());
    for id in ids {
        if id == s.user_id {
            skipped += 1;
            err = "err-self".into();
            continue;
        }
        match udb::get(&sc.db, id) {
            Ok(Some(u)) if u.role == "admin" && admins_left <= 1 => {
                skipped += 1;
                err = "err-last-admin".into();
            }
            Ok(Some(u)) => {
                if u.role == "admin" {
                    admins_left -= 1;
                }
                let _ = udb::delete(&sc.db, id);
                let _ = udb::session_delete_user(&sc.db, id);
                deleted += 1;
            }
            _ => {}
        }
    }
    let mut extra = if deleted > 0 {
        format!("msg=deleted&n={deleted}")
    } else if err.is_empty() {
        String::new()
    } else {
        format!("msg={err}")
    };
    if skipped > 0 {
        extra.push_str(&format!("&skipped={skipped}"));
    }
    if extra.is_empty() {
        return Redirect::to(&back).into_response();
    }
    redirect_with(&back, &extra)
}

/// users.php bulk: `action=delete` → GET confirm screen (WP two-step);
/// `changeit` + `new_role` → bulk "Change role to…". Guards mirror delete.
pub(crate) async fn users_bulk(State(app): State<App>, headers: HeaderMap, Form(map): Form<HashMap<String, String>>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = require_admin(&s) {
        return r.into_response();
    }
    if map.get("csrf").map(|x| x.as_str()) != Some(s.csrf.as_str()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let back = back_target(&map, "/admin/users");
    let ids = selected_ids(&map);
    if ids.is_empty() {
        return Redirect::to(&back).into_response();
    }
    // WP parity: bottom tablenav posts `action2`/`new_role2`; `action`
    // falling back to `action2` mirrors comments_bulk.
    let get = |k: &str| map.get(k).map(|s| s.as_str());
    let action = match get("action") { Some(a) if a != "-1" => a, _ => get("action2").unwrap_or("-1") };
    let want_role = map.contains_key("changeit");
    if !want_role && action == "delete" {
        return Redirect::to(&format!("/admin/users/delete?ids={}&back={}", ids.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(","), urlenc(&back))).into_response();
    }
    if !want_role {
        return Redirect::to(&back).into_response();
    }
    let new_role = match get("new_role") { Some(r) if !r.is_empty() => r, _ => get("new_role2").unwrap_or("") };
    if !matches!(new_role, "admin" | "editor") {
        return Redirect::to(&back).into_response();
    }
    // Track REMAINING admins inside the loop — a bulk request selecting all
    // admins must refuse the last one, not just an all-admin snapshot.
    let mut admins_left = udb::count_admins(&sc.db).unwrap_or(0);
    let (mut changed, mut skipped) = (0i64, 0i64);
    for id in ids {
        // Task rule: never change your own role via bulk.
        if id == s.user_id {
            skipped += 1;
            continue;
        }
        match udb::get(&sc.db, id) {
            Ok(Some(u)) => {
                // HARDENING: never leave zero admins — refuse when this
                // demotion would drop the count to 0.
                if u.role == "admin" && admins_left <= 1 && new_role != "admin" {
                    skipped += 1;
                    continue;
                }
                if u.role != new_role {
                    if u.role == "admin" && new_role != "admin" {
                        admins_left -= 1;
                    }
                    let _ = udb::change_role(&sc.db, id, new_role);
                    // Stale-role sessions die immediately (A9).
                    let _ = udb::session_delete_user(&sc.db, id);
                    drop_sessions_for(&app, id, None);
                    changed += 1;
                }
            }
            _ => {}
        }
    }
    let mut extra = if changed > 0 { format!("msg=roles&n={changed}") } else { "msg=err-roles".to_string() };
    if skipped > 0 {
        extra.push_str(&format!("&skipped={skipped}"));
    }
    redirect_with(&back, &extra)
}

/// `profile.php` — self-edit for any session; the Role row is omitted
/// entirely (WP parity), and `users_save` ignores `role` for self anyway.
pub(crate) async fn profile_page(State(app): State<App>, headers: HeaderMap, Query(q): Query<HashMap<String, String>>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else {
        return Redirect::to("/admin/login").into_response();
    };
    let Ok(Some(u)) = udb::get(&sc.db, s.user_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let notice = notice_html(q.get("msg").map(|s| s.as_str()).unwrap_or(""), 0, 0);
    let inner = user_edit_html(&s, &u, true, false, "/admin/profile", &notice);
    html_admin(&s, "Profile", &inner, true, "profile-php").into_response()
}

fn app_password() -> String {
    const C: &[u8] = b"abcdefghijklmnopqrstuvwxyz23456789";
    let mut b = [0u8; 24];
    rand::thread_rng().fill_bytes(&mut b);
    b.chunks(4)
        .map(|chunk| chunk.iter().map(|x| C[(*x as usize) % C.len()] as char).collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

/// POST `/admin/users/app-pass/add` — creates an Application Password,
/// shows the cleartext once, then stores only SHA-256.
pub(crate) async fn users_app_pass_add(State(app): State<App>, headers: HeaderMap, Form(f): Form<HashMap<String, String>>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else { return Redirect::to("/admin/login").into_response(); };
    if f.get("csrf").map(|x| x.as_str()) != Some(s.csrf.as_str()) { return StatusCode::FORBIDDEN.into_response(); }
    let user_id = f.get("user_id").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
    if user_id != s.user_id && require_admin(&s).is_err() { return StatusCode::FORBIDDEN.into_response(); }
    let name = f.get("name").map(|v| v.trim()).filter(|v| !v.is_empty()).unwrap_or("Application Password");
    let pw = app_password();
    let compact = pw.replace(' ', "");
    let hash = super::auth::sha256_hex(&compact);
    if udb::app_pass_insert(&sc.db, user_id, name, &hash).is_err() {
        return Redirect::to(&format!("/admin/user-edit/{user_id}?msg=err-app-pass")).into_response();
    }
    let Ok(Some(u)) = udb::get(&sc.db, user_id) else { return Redirect::to("/admin/users").into_response(); };
    let notice = format!("<div class=\"notice notice-success\"><p>Application password created. Copy it now; it will not be shown again: <code>{}</code></p></div>", esc(&pw));
    let inner = user_edit_html(&s, &u, false, false, "/admin/users", &notice);
    html_admin(&s, "Edit User", &inner, true, "user-edit-php").into_response()
}

/// POST `/admin/users/app-pass/revoke` — deletes one app password for a user.
pub(crate) async fn users_app_pass_revoke(State(app): State<App>, headers: HeaderMap, Form(f): Form<HashMap<String, String>>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else { return Redirect::to("/admin/login").into_response(); };
    if f.get("csrf").map(|x| x.as_str()) != Some(s.csrf.as_str()) { return StatusCode::FORBIDDEN.into_response(); }
    let user_id = f.get("user_id").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
    if user_id != s.user_id && require_admin(&s).is_err() { return StatusCode::FORBIDDEN.into_response(); }
    let id = f.get("id").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
    let _ = udb::app_pass_revoke(&sc.db, id, user_id);
    Redirect::to(&format!("/admin/user-edit/{user_id}?msg=updated")).into_response()
}
