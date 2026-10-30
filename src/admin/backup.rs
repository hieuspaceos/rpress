//! `/admin/tools/backup` — create + download site backup archives
//! (`work/<site>/backups/*.tar.gz`). Core work lives in `crate::backup`;
//! this is the thin HTTP shell: page, "Backup now" POST, basename-guarded
//! download.

use super::{check_csrf, esc, need_auth, App, SiteCtx};
use axum::{
    body::Body,
    extract::{Form, Path as AxPath, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

fn backups_dir(sc: &SiteCtx) -> std::path::PathBuf {
    sc.db.parent().unwrap_or_else(|| std::path::Path::new(".")).join("backups")
}

/// GET — list archives + "Backup now" button.
pub(crate) async fn backup_page(State(app): State<App>, headers: HeaderMap) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else { return Redirect::to("/admin/login").into_response(); };
    let dir = backups_dir(sc);
    let files = crate::backup::list(&dir);
    let mut rows = String::new();
    for (name, size) in &files {
        let mb = *size as f64 / 1_048_576.0;
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{:.1} MB</td><td><a href=\"/admin/tools/backup/download/{}\">Download</a></td></tr>",
            esc(name), mb, esc(name)
        ));
    }
    let table = if rows.is_empty() {
        "<p>No backups yet.</p>".to_string()
    } else {
        format!("<table class=\"wp-list-table widefat striped\"><thead><tr><th>Archive</th><th>Size</th><th></th></tr></thead><tbody>{rows}</tbody></table>")
    };
    let body = format!(
        r#"<h1>Backup</h1><hr class="wp-header-end">
<p>Snapshots the site workdir into <code>work/{}/backups/&lt;timestamp&gt;.tar.gz</code> — a consistent <code>VACUUM INTO</code> copy of <code>rpress.db</code> plus <code>www/</code> and <code>sites/{}.env</code>. Keeps the newest 5 archives. Restore via <code>rpress site init --backup &lt;archive&gt;</code>.</p>
<form method="post" action="/admin/tools/backup"><input type="hidden" name="csrf" value="{}"><button type="submit" class="button button-primary">Backup now</button></form>
{table}"#,
        esc(&sc.slug), esc(&sc.slug), esc(&s.csrf)
    );
    Html(format!("{}{}{}", super::chrome("tools", &s), body, super::chrome_end(&s))).into_response()
}

#[derive(Deserialize)]
pub(crate) struct CsrfForm {
    csrf: String,
}

/// POST — run the backup synchronously, redirect back.
pub(crate) async fn backup_run(State(app): State<App>, headers: HeaderMap, Form(f): Form<CsrfForm>) -> Response {
    let Ok((s, sc)) = need_auth(&app, &headers) else { return Redirect::to("/admin/login").into_response(); };
    if let Err(r) = check_csrf(&s, &f.csrf) { return r; }
    let work = sc.db.parent().unwrap_or_else(|| std::path::Path::new(".")).to_path_buf();
    let env = crate::site::env_path(&std::env::current_dir().unwrap_or_default(), &sc.slug);
    match crate::backup::run_backup(&work, &env, 5) {
        Ok(_) => Redirect::to("/admin/tools/backup").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("backup failed: {e}")).into_response(),
    }
}

/// GET download — basename only (`backup::resolve` guards traversal).
pub(crate) async fn backup_download(State(app): State<App>, headers: HeaderMap, AxPath(name): AxPath<String>) -> Response {
    let Ok((_s, sc)) = need_auth(&app, &headers) else { return Redirect::to("/admin/login").into_response(); };
    let dir = backups_dir(sc);
    let Ok(path) = crate::backup::resolve(&dir, &name) else { return StatusCode::BAD_REQUEST.into_response() };
    // Stream in chunks — a multi-GB archive can't be read into memory.
    let Ok(mut file) = std::fs::File::open(&path) else { return StatusCode::NOT_FOUND.into_response(); };
    let stream = futures_util::stream::unfold(file, |mut f| async move {
        let chunk = tokio::task::spawn_blocking(move || {
            let mut buf = vec![0u8; 256 * 1024];
            let n = std::io::Read::read(&mut f, &mut buf).unwrap_or(0);
            buf.truncate(n);
            (buf, f)
        }).await.ok();
        chunk.and_then(|(buf, f)| {
            if buf.is_empty() { None } else { Some((Ok::<_, std::io::Error>(axum::body::Bytes::from(buf)), f)) }
        })
    });
    Response::builder()
        .header(header::CONTENT_TYPE, "application/gzip")
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{name}\""))
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}


