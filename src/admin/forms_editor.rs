//! CF7-style form editor — `/admin/contact/form/{slug}/edit`.
//!
//! Four tabs (Form | Mail | Messages | Additional Settings) matching
//! `admin.php?page=wpcf7&action=edit`. Save goes to `forms` table; intake
//! reads config from DB at submit time.
use super::*;
use crate::db::forms as fdb;
use crate::db::forms::FormRow;
use crate::forms_markup;
use axum::extract::{Form, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use std::collections::HashMap;

const TAG_TYPES: &[&str] = &[
    "text", "email", "url", "tel", "number", "date", "textarea", "select",
    "checkbox", "radio", "acceptance", "quiz", "file", "submit",
];

const MSG_KEYS: &[(&str, &str)] = &[
    ("success", "Sender's message was sent successfully"),
    ("mail_sent_ng", "Sender's message failed to send"),
    ("validation", "Validation errors occurred"),
    ("spam", "Submission was referred to as spam"),
    ("accept_terms", "There are terms that the sender must accept"),
    ("required", "There is a field that the sender must fill in"),
    ("email_invalid", "Email address that the sender entered is invalid"),
    ("url_invalid", "URL that the sender entered is invalid"),
    ("tel_invalid", "Telephone number that the sender entered is invalid"),
    ("date_invalid", "Date format that the sender entered is invalid"),
    ("number_invalid", "Number format that the sender entered is invalid"),
    ("file_too_big", "Uploaded file is too large"),
    ("file_type", "Uploaded file is not allowed for file type"),
    ("quiz_wrong", "Sender doesn't enter the correct answer to the quiz"),
    ("upload_failed", "Uploading a file fails for any reason"),
];

/// Tab bar for the edit page — mimics CF7's `.nav-tab-wrapper`.
fn tabs(slug: &str, active: &str) -> String {
    let mk = |key: &str, label: &str| {
        let cls = if key == active { "nav-tab nav-tab-active" } else { "nav-tab" };
        format!("<a class=\"{cls}\" href=\"/admin/contact/form/{}/edit?tab={key}\">{label}</a>", esc(slug))
    };
    format!(
        "<nav class=\"nav-tab-wrapper wp-clearfix\">{}{}{}{}</nav>",
        mk("form", "Form"),
        mk("mail", "Mail"),
        mk("messages", "Messages"),
        mk("additional", "Additional Settings"),
    )
}

/// Tag-generator buttons row + modal markup (hidden until a button is
/// clicked). JS in `admin.js` handles open/insert; this is the static
/// markup we always ship.
fn tag_panel() -> String {
    let mut btns = String::new();
    for t in TAG_TYPES {
        btns.push_str(&format!(
            "<button type=\"button\" class=\"button\" data-tagtype=\"{t}\" onclick=\"return cf7_tag('{t}')\">{t}</button> "
        ));
    }
    format!(
        r#"<div id="tag-generator" class="control-box">
<fieldset><legend>Generate a form-tag for a</legend>
<p>{btns}</p>
<div id="cf7-tag-panel" class="hidden">
  <h3 id="cf7-tag-panel-title"></h3>
  <table class="form-table"><tbody id="cf7-tag-fields"></tbody></table>
  <p>
    <input type="text" id="cf7-tag-result" class="widefat code" readonly>
    <button type="button" class="button button-primary" onclick="return cf7_insert()">Insert Tag</button>
    <button type="button" class="button" onclick="return cf7_close()">Cancel</button>
  </p>
</div>
</fieldset>
</div>
<script>
const CF7_TYPES = {{
  text:      {{name:true,  opts:['required','default','placeholder','id','class']}},
  email:     {{name:true,  opts:['required','default','placeholder','id','class']}},
  url:       {{name:true,  opts:['required','default','placeholder','id','class']}},
  tel:       {{name:true,  opts:['required','default','placeholder','id','class']}},
  number:    {{name:true,  opts:['required','default','min','max','placeholder','id','class']}},
  date:      {{name:true,  opts:['required','default','min','max','id','class']}},
  textarea:  {{name:true,  opts:['required','default','placeholder','id','class','cols','rows']}},
  select:    {{name:true,  opts:['required','options','multiple','first_as_label','id','class']}},
  checkbox:  {{name:true,  opts:['required','options','exclusive','id','class']}},
  radio:     {{name:true,  opts:['required','options','default','id','class']}},
  acceptance:{{name:true,  opts:['invert','id','class']}},
  quiz:      {{name:true,  opts:['required','question','answer','id','class']}},
  file:      {{name:true,  opts:['required','limit','accept','id','class']}},
  submit:    {{name:false, opts:['label','id','class']}},
}};
let CF7_ACTIVE = null;
function cf7_tag(t){{
  CF7_ACTIVE = t;
  const cfg = CF7_TYPES[t];
  document.getElementById('cf7-tag-panel-title').textContent = 'Tag generator — ' + t;
  const tb = document.getElementById('cf7-tag-fields');
  tb.innerHTML = '';
  if (cfg.name) {{
    tb.innerHTML += '<tr><th><label>Name</label></th><td><input id="cf7f-name" class="regular-text"></td></tr>';
  }}
  for (const o of cfg.opts) {{
    const id = 'cf7f-' + o;
    if (o === 'required') {{
      tb.innerHTML += '<tr><td colspan="2"><label><input type="checkbox" id="' + id + '"> Required field</label></td></tr>';
    }} else if (o === 'options' || o === 'question' || o === 'answer' || o === 'label' || o === 'default' || o === 'placeholder' || o === 'first_as_label' || o === 'invert' || o === 'exclusive' || o === 'multiple') {{
      tb.innerHTML += '<tr><th><label>' + o + '</label></th><td><input id="' + id + '" class="regular-text"></td></tr>';
    }} else {{
      tb.innerHTML += '<tr><th><label>' + o + '</label></th><td><input id="' + id + '" class="small-text"></td></tr>';
    }}
  }}
  document.getElementById('cf7-tag-panel').classList.remove('hidden');
  cf7_update();
  return false;
}}
function cf7_close(){{
  document.getElementById('cf7-tag-panel').classList.add('hidden');
  CF7_ACTIVE = null;
  return false;
}}
function cf7_update(){{
  if (!CF7_ACTIVE) return;
  const cfg = CF7_TYPES[CF7_ACTIVE];
  const g = id => document.getElementById('cf7f-' + id);
  let tag = CF7_ACTIVE;
  if (cfg.name && g('required') && g('required').checked) tag += '*';
  if (cfg.name) tag += ' ' + (g('name')?.value || 'name');
  for (const o of cfg.opts) {{
    const el = g(o); if (!el) continue;
    if (el.type === 'checkbox') continue;
    const v = el.value.trim(); if (!v) continue;
    if (o === 'default') tag += ' default:"' + v + '"';
    else if (o === 'placeholder') tag += ' placeholder:"' + v + '"';
    else if (o === 'label' && CF7_ACTIVE === 'submit') tag += ' "' + v + '"';
    else if (o === 'options' && ['select','checkbox','radio'].includes(CF7_ACTIVE)) tag += ' ' + v.split(',').map(s => '"' + s.trim() + '"').join(' ');
    else tag += ' ' + o + ':' + v;
  }}
  tag += ']';
  tag = '[' + tag.replace(/^\\[|\\]$/g, '');
  document.getElementById('cf7-tag-result').value = tag;
}}
function cf7_insert(){{
  const v = document.getElementById('cf7-tag-result').value;
  const ta = document.getElementById('cf7-markup');
  const pos = ta.selectionStart || ta.value.length;
  ta.value = ta.value.slice(0, pos) + v + ta.value.slice(pos);
  ta.focus();
  return cf7_close();
}}
document.addEventListener('input', e => {{
  if (e.target.id && e.target.id.startsWith('cf7f-')) cf7_update();
}});
</script>"#
    )
}

/// GET `/admin/contact/form/{slug}/edit` — the CF7-style edit page.
pub(crate) async fn form_edit(
    State(app): State<App>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    let tab = q.get("tab").cloned().unwrap_or_else(|| "form".to_string());
    let notice = if q.get("saved").is_some() {
        "<div class=\"notice notice-success is-dismissible\"><p>Form saved.</p></div>".to_string()
    } else if let Some(err) = q.get("error") {
        format!("<div class=\"notice notice-error\"><p>{}</p></div>", esc(err))
    } else {
        String::new()
    };
    // rusqlite is sync — row fetch and the chrome render (comments counts,
    // kind probes) ride one blocking thread (audit-261031 C1).
    let (s, sc_c, slug2) = (s, sc.clone(), slug.clone());
    tokio::task::spawn_blocking(move || {
        let Ok(Some(r)) = fdb::get_by_slug(&sc_c.db, &slug2) else {
            return StatusCode::NOT_FOUND.into_response();
        };

        let content = match tab.as_str() {
            "mail" => mail_tab(&r),
            "messages" => messages_tab(&r),
            "additional" => additional_tab(&r),
            _ => form_tab(&r),
        };

        let body = format!(
            r#"{}<div class="wrap">
<h1 class="wp-heading-inline">Contact Form</h1>
<span class="subtitle">{slug}</span>
<hr class="wp-header-end">{notice}
{tabs}
<form method="post" action="/admin/contact/form/{slug}/save">
<input type="hidden" name="csrf" value="{}">
<input type="hidden" name="tab" value="{tab}">
{content}
<p class="submit">
  <input type="submit" class="button button-primary" value="Save">
  <a class="button" href="/admin/contact">Back to list</a>
</p>
</form>
</div>"#,
            chrome("contact", &s),
            s.csrf,
            slug = esc(&slug),
            tab = esc(&tab),
            tabs = tabs(&slug, &tab),
            content = content,
            notice = notice,
        );
        html_admin(&s, &format!("Contact Form ‹ {}", s.site_name), &body, true, "edit-php").into_response()
    })
    .await
    .unwrap_or_else(|e| {
        eprintln!("form_edit join: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })
}

fn form_tab(r: &FormRow) -> String {
    format!(
        r#"{panel}
<table class="form-table"><tbody>
<tr><th><label for="f-title">Title</label></th>
  <td><input id="f-title" name="title" class="regular-text" value="{title}"></td></tr>
<tr><th><label for="cf7-markup">Form</label></th>
  <td><textarea id="cf7-markup" name="markup" class="large-text code" rows="24" cols="100">{markup}</textarea>
  <p class="description">Use CF7-style tags: <code>[text* your-name]</code>, <code>[textarea* your-message]</code>, <code>[submit "Send"]</code>. See the tag generator above.</p></td></tr>
</tbody></table>"#,
        panel = tag_panel(),
        title = esc(&r.title),
        markup = esc(&r.markup),
    )
}

fn mail_tab(r: &FormRow) -> String {
    format!(
        r#"<table class="form-table"><tbody>
<tr><th><label>To</label></th><td><input name="mail_to" class="large-text" value="{}"></td></tr>
<tr><th><label>From</label></th><td><input name="mail_from" class="large-text" value="{}"></td></tr>
<tr><th><label>Subject</label></th><td><input name="mail_subject" class="large-text" value="{}"></td></tr>
<tr><th><label>Message body</label></th><td><textarea name="mail_body" class="large-text code" rows="10">{}</textarea></td></tr>
<tr><th><label>Additional headers</label></th><td><textarea name="mail_headers" class="large-text code" rows="3"></textarea><p class="description">Cc:, Bcc:, Reply-To: — one per line.</p></td></tr>
<tr><td colspan="2"><label><input type="checkbox" name="mail_html" value="1"> Use HTML content type</label></td></tr>
<tr><td colspan="2"><hr></td></tr>
<tr><td colspan="2"><label><input type="checkbox" name="mail2_active" value="1" {}> <strong>Mail (2)</strong> — activate secondary mail</label></td></tr>
<tr><th><label>To (2)</label></th><td><input name="mail2_to" class="large-text" value="{}"></td></tr>
<tr><th><label>Subject (2)</label></th><td><input name="mail2_subject" class="large-text" value="{}"></td></tr>
<tr><th><label>Message body (2)</label></th><td><textarea name="mail2_body" class="large-text code" rows="10">{}</textarea></td></tr>
</tbody></table>"#,
        esc(&r.mail_to),
        esc(&r.mail_from),
        esc(&r.mail_subject),
        esc(&r.mail_body),
        if r.mail2_active { "checked" } else { "" },
        esc(&r.mail2_to),
        esc(&r.mail2_subject),
        esc(&r.mail2_body),
    )
}

fn messages_tab(r: &FormRow) -> String {
    let msgs: serde_json::Value =
        serde_json::from_str(&r.messages).unwrap_or_else(|_| serde_json::json!({}));
    let mut rows = String::new();
    for (key, label) in MSG_KEYS {
        let v = msgs.get(*key).and_then(|v| v.as_str()).unwrap_or("");
        rows.push_str(&format!(
            "<tr><th><label for=\"m-{key}\">{label}</label></th>\
             <td><input id=\"m-{key}\" name=\"msg_{key}\" class=\"large-text\" value=\"{}\"></td></tr>",
            esc(v)
        ));
    }
    format!(
        "<table class=\"form-table\"><tbody>{rows}</tbody></table>\
         <p class=\"description\">Mail delivery is not configured in this build — messages are used for the on-screen response only.</p>"
    )
}

fn additional_tab(r: &FormRow) -> String {
    format!(
        r#"<table class="form-table"><tbody>
<tr><th><label for="additional">Additional Settings</label></th>
  <td><textarea id="additional" name="additional" class="large-text code" rows="10">{}</textarea>
  <p class="description">One <code>key: value</code> pair per line. E.g. <code>flamingo: on</code>, <code>honeypot: my-gotcha</code>.</p></td></tr>
</tbody></table>"#,
        esc(&r.additional)
    )
}

#[derive(Deserialize)]
#[allow(dead_code)] // full CF7 save payload; tabs saved incrementally
pub(crate) struct SaveForm {
    #[serde(default)]
    csrf: String,
    tab: Option<String>,
    title: Option<String>,
    markup: Option<String>,
    mail_to: Option<String>,
    mail_from: Option<String>,
    mail_subject: Option<String>,
    mail_body: Option<String>,
    mail2_active: Option<String>,
    mail2_to: Option<String>,
    mail2_subject: Option<String>,
    mail2_body: Option<String>,
    additional: Option<String>,
}

/// POST `/admin/contact/form/{slug}/save` — writes the tab's fields.
pub(crate) async fn form_save(
    State(app): State<App>,
    headers: HeaderMap,
    Path(slug): Path<String>,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    // CSRF via the same session path as every other handler — the old
    // hand-rolled cookie→map lookup bypassed get_sess's DB rehydration and
    // 403'd mid-form after a restart (M16).
    if let Err(r) = check_csrf(&s, f.get("csrf").map(|x| x.as_str()).unwrap_or("")) {
        return r;
    }

    // rusqlite is sync — offload the row fetch to a blocking thread.
    let db = sc.db.clone();
    let slug2 = slug.clone();
    let r = tokio::task::spawn_blocking(move || fdb::get_by_slug(&db, &slug2))
        .await
        .unwrap_or_else(|_| Err(anyhow::anyhow!("join")));
    let Ok(Some(mut r)) = r else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let tab = f.get("tab").map(|s| s.as_str()).unwrap_or("form");

    // Apply the posted fields onto the row.
    if let Some(v) = f.get("title") { r.title = v.clone(); }
    if let Some(v) = f.get("markup") { r.markup = v.clone(); }
    if let Some(v) = f.get("mail_to") { r.mail_to = v.clone(); }
    if let Some(v) = f.get("mail_from") { r.mail_from = v.clone(); }
    if let Some(v) = f.get("mail_subject") { r.mail_subject = v.clone(); }
    if let Some(v) = f.get("mail_body") { r.mail_body = v.clone(); }
    r.mail2_active = f.get("mail2_active").is_some();
    if let Some(v) = f.get("mail2_to") { r.mail2_to = v.clone(); }
    if let Some(v) = f.get("mail2_subject") { r.mail2_subject = v.clone(); }
    if let Some(v) = f.get("mail2_body") { r.mail2_body = v.clone(); }
    if let Some(v) = f.get("additional") { r.additional = v.clone(); }

    // Merge posted `msg_*` keys into the messages JSON.
    let mut msgs: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&r.messages).unwrap_or_default();
    for (k, _) in MSG_KEYS {
        if let Some(v) = f.get(&format!("msg_{k}")) {
            msgs.insert((*k).into(), serde_json::Value::String(v.clone()));
        }
    }
    r.messages = serde_json::to_string(&msgs).unwrap_or_else(|_| "{}".into());

    // Validate markup + mail refs (only on the tabs that touch them).
    if tab == "form" || tab == "mail" {
        if let Err(e) = forms_markup::validate(
            &r.markup, &r.mail_to, &r.mail_from, &r.mail_subject, &r.mail_body,
        ) {
            let enc = urlenc(&e);
            return Redirect::to(&format!(
                "/admin/contact/form/{slug}/edit?tab={tab}&error={enc}"
            ))
            .into_response();
        }
    }

    // rusqlite is sync — offload the forms UPDATE to a blocking thread.
    let db = sc.db.clone();
    let saved = tokio::task::spawn_blocking(move || {
        fdb::update(
            &db, r.id, &r.title, &r.markup, &r.mail_to, &r.mail_from,
            &r.mail_subject, &r.mail_body, r.mail2_active, &r.mail2_to,
            &r.mail2_subject, &r.mail2_body, &r.messages, &r.additional,
        )
        .is_ok()
    })
    .await
    .unwrap_or(false);
    if !saved {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    Redirect::to(&format!("/admin/contact/form/{slug}/edit?tab={tab}&saved=1")).into_response()
}

// Percent-encoding for redirect params uses the shared byte-wise `urlenc`
// (mod.rs) — the old codepoint-based copy produced mojibake on non-ASCII
// error text (M16).

/// GET `/admin/contact/new` — create a new contact form.
pub(crate) async fn form_new(State(app): State<App>, headers: HeaderMap, Query(q): Query<HashMap<String, String>>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    let notice = q
        .get("error")
        .map(|e| format!("<div class=\"notice notice-error\"><p>{}</p></div>", esc(e)))
        .unwrap_or_default();
    // Language options come from the site's lang config — N-language
    // sites get one option per configured lang, never fr/es literals.
    // rusqlite + chrome's DB reads are sync — lang config and the render
    // ride one blocking thread (audit-261031 C1).
    let (s, db) = (s, sc.db.clone());
    tokio::task::spawn_blocking(move || {
        let lang_opts = match crate::db::conn(&db) {
            Ok(c) => {
                let cfg = crate::lang::config(&c);
                cfg.langs
                    .iter()
                    .map(|l| format!("<option value=\"{}\">{}</option>", esc(l), esc(&crate::lang::lang_label(l))))
                    .collect::<Vec<_>>()
                    .join("")
            }
            Err(_) => String::new(),
        };
        let body = format!(
            r#"{}<div class="wrap"><h1>Add New Contact Form</h1>
{notice}
<form method="post" action="/admin/contact/create">
<input type="hidden" name="csrf" value="{}">
<table class="form-table"><tbody>
<tr><th><label>Title</label></th><td><input name="title" class="regular-text" required></td></tr>
<tr><th><label>Slug</label></th><td><input name="slug" class="regular-text code" required>
  <p class="description">URL-friendly identifier — used in <code>/form/&lt;slug&gt;</code> endpoint.</p></td></tr>
<tr><th><label>Language</label></th><td><select name="lang">{lang_opts}</select></td></tr>
<tr><th><label>Kind</label></th><td><select name="kind"><option value="form">Contact form</option><option value="review">Site review</option></select></td></tr>
</tbody></table>
<p class="submit"><input type="submit" class="button button-primary" value="Create"></p>
</form></div>"#,
            chrome("contact", &s),
            s.csrf,
            notice = notice,
        );
        html_admin(&s, "Add Contact Form", &body, true, "edit-php").into_response()
    })
    .await
    .unwrap_or_else(|e| {
        eprintln!("form_new join: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    })
}

/// POST `/admin/contact/create` — create + redirect to edit.
pub(crate) async fn form_create(
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
    let slug = f.get("slug").cloned().unwrap_or_default();
    let title = f.get("title").cloned().unwrap_or_default();
    // rusqlite is sync — the lang fallback reads site config off the
    // async thread.
    let db = sc.db.clone();
    let lang = match f.get("lang") {
        Some(v) => v.clone(),
        None => tokio::task::spawn_blocking(move || {
            crate::db::conn(&db)
                .map(|c| crate::lang::config(&c).primary)
                .unwrap_or_default()
        })
        .await
        .unwrap_or_default(),
    };
    let kind = f.get("kind").cloned().unwrap_or_else(|| "form".into());
    // Slug becomes a URL path segment + UNIQUE column: reject anything
    // outside [a-z0-9-] and duplicates, with a visible error instead of a
    // silent list bounce (M16).
    let valid_slug = !slug.is_empty()
        && slug.len() <= 64
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !valid_slug {
        return Redirect::to(&format!(
            "/admin/contact/new?error={}",
            urlenc("Invalid slug — use lowercase letters, digits and dashes only.")
        ))
        .into_response();
    }
    if title.is_empty() {
        return Redirect::to("/admin/contact").into_response();
    }
    let default_markup = "<label> Your name\n    [text* your-name] </label>\n\
        <label> Your email\n    [email* your-email] </label>\n\
        <label> Your message\n    [textarea* your-message] </label>\n\
        [submit \"Send\"]\n";
    let msgs = r#"{"success":"Thank you for your message. It has been sent."}"#;
    let body = "From: [your-name] <[your-email]>\n\n[your-message]\n";
    // rusqlite is sync — offload the INSERT to a blocking thread.
    let db = sc.db.clone();
    let slug2 = slug.clone();
    let inserted = tokio::task::spawn_blocking(move || {
        fdb::insert(&db, &slug2, &title, &lang, &kind, default_markup, "", "[your-email]", "", body, msgs)
            .is_ok()
    })
    .await
    .unwrap_or(false);
    if !inserted {
        return Redirect::to(&format!(
            "/admin/contact/new?error={}",
            urlenc(&format!("A form named '{slug}' already exists."))
        ))
        .into_response();
    }
    Redirect::to(&format!("/admin/contact/form/{slug}/edit")).into_response()
}
