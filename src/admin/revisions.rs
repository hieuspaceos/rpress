//! WP `revision.php`: per-post revision list, side-by-side diff vs the
//! current content, and restore. Snapshots are taken on every save
//! (`posts__insert_into_revisions`); restore writes a new snapshot first
//! like WP does, so a restore is itself undoable.

use super::*;

/// `GET /admin/posts/{id}/revisions` — list of snapshots, newest first.
pub(crate) async fn revisions_page(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    // rusqlite is sync — offload conn+queries (and the page render built
    // on them) to a blocking thread (audit P1).
    let (s2, sc2) = (s.clone(), sc.clone());
    return tokio::task::spawn_blocking(move || revisions_page_sync(&s2, &sc2, id))
        .await
        .unwrap_or_else(|e| {
            eprintln!("revisions_page join: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        });
}

fn revisions_page_sync(s: &Sess, sc: &SiteCtx, id: i64) -> Response { // gate-allow: blocking by contract — only called inside spawn_blocking
    let Some(p) = crud::load_post(&sc.db, id).ok().flatten() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let revs = crud::revisions_for(&sc.db, id).unwrap_or_default();
    let mut rows = String::new();
    for (rid, saved, len) in &revs {
        rows.push_str(&format!(
            r##"<tr><td>{}</td><td>{} bytes</td>
<td class="row-actions"><a href="/admin/revisions/{rid}">Compare</a> | <form method="post" action="/admin/revisions/{rid}/restore" style="display:inline" onsubmit="return confirm('Restore this revision? The current content is snapshotted first.')"><input type="hidden" name="csrf" value="{csrf}"><button class="button-link">Restore</button></form></td></tr>"##,
            esc(saved),
            len,
            rid = rid,
            csrf = s.csrf,
        ));
    }
    if rows.is_empty() {
        rows.push_str(r#"<tr class="no-items"><td colspan="3">No revisions yet — one is captured on each save.</td></tr>"#);
    }
    let kind_label = posts::spec_for_kind(&p.kind).map(|k| k.noun).unwrap_or(&p.kind);
    let body = format!(
        r#"<div class="wrap"><h1 class="wp-heading-inline">Revisions for “{}”</h1>
<a href="/admin/edit/{id}" class="page-title-action">Back to {kind_label}</a>
<hr class="wp-header-end">
<table class="wp-list-table widefat fixed striped">
<thead><tr><th scope="col">Saved</th><th scope="col">Size</th><th scope="col">Actions</th></tr></thead>
<tbody>{rows}</tbody></table>
<p class="description">Each save snapshots title + content. Restore copies a revision back — the current content is snapshotted first, so restores are themselves undoable.</p></div>"#,
        esc(&p.title),
    );
    html_page(&s, "Revisions", &format!("{}{}", chrome(posts::spec_for_kind(&p.kind).map(|k| k.section).unwrap_or("posts"), &s), body), false).into_response()
}

/// `GET /admin/revisions/{rid}` — diff the revision against the *current*
/// post content (WP's comparison slider pinned at left=revision,
/// right=current). Title and body are compared separately; body diffs run
/// on the rendered HTML (revision blobs store body_json trees, and the
/// list shows text the author recognizes).
pub(crate) async fn revision_diff(
    State(app): State<App>,
    headers: HeaderMap,
    Path(rid): Path<i64>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    // rusqlite is sync — offload conn+queries (and the diff render) to a
    // blocking thread (audit P1).
    let (s2, sc2) = (s.clone(), sc.clone());
    return tokio::task::spawn_blocking(move || revision_diff_sync(&s2, &sc2, rid))
        .await
        .unwrap_or_else(|e| {
            eprintln!("revision_diff join: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        });
}

fn revision_diff_sync(s: &Sess, sc: &SiteCtx, rid: i64) -> Response { // gate-allow: blocking by contract — only called inside spawn_blocking
    let Some((post_id, saved, r_title, _r_excerpt, r_body)) =
        crud::revision_get(&sc.db, rid).ok().flatten()
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(p) = crud::load_post(&sc.db, post_id).ok().flatten() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let cur_html = crud::html_from_body_json(&p.body_json);
    let rev_html = crud::html_from_body_json(&r_body);
    let title_diff = line_diff(&r_title, &p.title);
    let body_diff = line_diff(&rev_html, &cur_html);
    let restore = format!(
        r##"<form method="post" action="/admin/revisions/{rid}/restore" style="display:inline"><input type="hidden" name="csrf" value="{csrf}"><button class="button button-primary" onclick="return confirm('Restore this revision? The current content is snapshotted first.')">Restore This Revision</button></form>"##,
        csrf = s.csrf
    );
    let body = format!(
        r#"<div class="wrap"><h1 class="wp-heading-inline">Compare Revisions of “{}”</h1>
<a href="/admin/posts/{post_id}/revisions" class="page-title-action">Back to list</a>
<hr class="wp-header-end">
<p>Comparing revision saved <b>{}</b> (left/removed) against the <b>current</b> content (right/added). {restore}</p>
<style>
table.rev-diff{{width:100%;border-collapse:collapse;font-family:Consolas,Monaco,monospace;font-size:12px}}
table.rev-diff td{{border:1px solid #dcdcde;padding:4px 8px;vertical-align:top;word-break:break-word}}
td.rev-del{{background:#fcf0f1}}
td.rev-ins{{background:#edfaef}}
td.rev-del del{{background:#ffb8bb;text-decoration:none}}
td.rev-ins ins{{background:#abdfad;text-decoration:none}}
</style>
<h3>Title</h3><table class="rev-diff">{title_diff}</table>
<h3>Content</h3><table class="rev-diff">{body_diff}</table></div>"#,
        esc(&p.title),
        esc(&saved),
    );
    html_page(&s, "Compare Revisions", &format!("{}{}", chrome(posts::spec_for_kind(&p.kind).map(|k| k.section).unwrap_or("posts"), &s), body), false).into_response()
}

/// `POST /admin/revisions/{rid}/restore` — snapshot current, copy the
/// revision's title/excerpt/body back (WP `_wp_restore_post_revision`).
pub(crate) async fn revision_restore(
    State(app): State<App>,
    headers: HeaderMap,
    Path(rid): Path<i64>,
    Form(f): Form<HashMap<String, String>>,
) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers).await else {
        return Redirect::to("/admin/login").into_response();
    };
    if let Err(r) = check_csrf(&s, f.get("csrf").map(|v| v.as_str()).unwrap_or("")) {
        return r;
    }
    // rusqlite is sync — offload the snapshot+restore writes to a
    // blocking thread (audit P1); a join failure reports like the
    // original Err branch.
    let db = sc.db.clone();
    match tokio::task::spawn_blocking(move || crud::restore_revision(&db, rid))
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("restore join: {e:#}")))
    {
        Ok(post_id) => Redirect::to(&format!("/admin/posts/{post_id}/revisions")).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, format!("{e}")).into_response(),
    }
}

/// Split rendered HTML into diff-friendly lines: one element per line.
/// body_json text-box content is raw WP HTML; `><` joins are where tags
/// meet text, so splitting there gives stable granularity without pulling
/// an HTML parser in.
fn diff_lines(html: &str) -> Vec<String> {
    html.replace("><", ">\n<")
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Line diff between `old` (revision) and `new` (current), with a word
/// diff inside changed line pairs — WP's ins/del styling.
fn line_diff(old: &str, new: &str) -> String {
    use similar::{ChangeTag, TextDiff};
    let old_s = diff_lines(old).join("\n");
    let new_s = diff_lines(new).join("\n");
    let diff = TextDiff::from_lines(&old_s, &new_s);
    let mut out = String::new();
    // Collect changes per group so a del-block followed by an ins-block can
    // be word-diffed pairwise (the common "edited one line" case).
    for group in diff.grouped_ops(3) {
        for op in &group {
            match op.tag() {
                similar::DiffTag::Equal => {
                    for ch in diff.iter_changes(op) {
                        if ch.tag() == ChangeTag::Equal {
                            out.push_str(&format!("<tr><td>{}</td></tr>", esc(&ch.to_string())));
                        }
                    }
                }
                similar::DiffTag::Delete => {
                    for ch in diff.iter_changes(op) {
                        if ch.tag() == ChangeTag::Delete {
                            out.push_str(&format!("<tr><td class=\"rev-del\">{}</td></tr>", esc(&ch.to_string())));
                        }
                    }
                }
                similar::DiffTag::Insert => {
                    for ch in diff.iter_changes(op) {
                        if ch.tag() == ChangeTag::Insert {
                            out.push_str(&format!("<tr><td class=\"rev-ins\">{}</td></tr>", esc(&ch.to_string())));
                        }
                    }
                }
                similar::DiffTag::Replace => {
                    // Word-level diff of the replace block: pair each
                    // deleted line with its inserted counterpart.
                    let mut dels: Vec<String> = Vec::new();
                    let mut inss: Vec<String> = Vec::new();
                    for ch in diff.iter_changes(op) {
                        match ch.tag() {
                            ChangeTag::Delete => dels.push(ch.to_string()),
                            ChangeTag::Insert => inss.push(ch.to_string()),
                            _ => {}
                        }
                    }
                    let pairs = dels.len().max(inss.len());
                    for i in 0..pairs {
                        let (d, ins) = (
                            dels.get(i).cloned().unwrap_or_default(),
                            inss.get(i).cloned().unwrap_or_default(),
                        );
                        let (dh, ih) = word_diff(&d, &ins);
                        if !d.is_empty() {
                            out.push_str(&format!("<tr><td class=\"rev-del\">{dh}</td></tr>"));
                        }
                        if !ins.is_empty() {
                            out.push_str(&format!("<tr><td class=\"rev-ins\">{ih}</td></tr>"));
                        }
                    }
                }
            }
        }
    }
    out
}

/// Word diff of one line pair → (old-side html with <del>, new-side with
/// <ins>), each change escaped.
fn word_diff(old: &str, new: &str) -> (String, String) {
    use similar::{ChangeTag, TextDiff};
    let d = TextDiff::from_words(old, new);
    let (mut o, mut n) = (String::new(), String::new());
    for ch in d.iter_all_changes() {
        match ch.tag() {
            ChangeTag::Delete => o.push_str(&format!("<del>{}</del>", esc(&ch.to_string()))),
            ChangeTag::Insert => n.push_str(&format!("<ins>{}</ins>", esc(&ch.to_string()))),
            ChangeTag::Equal => {
                o.push_str(&esc(&ch.to_string()));
                n.push_str(&esc(&ch.to_string()));
            }
        }
    }
    (o, n)
}
