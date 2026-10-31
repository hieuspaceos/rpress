//! GoodLayers media endpoint used by the wp.media shim.

use super::*;
/// GET /api/v1/gdlr/media — id/path/alt list for the upload field picker.
pub(crate) async fn media_list(State(app): State<App>, headers: HeaderMap) -> Response {
    let Ok((_, sc)) = need_auth(&app, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    // rusqlite is sync — offload the media query to a blocking thread.
    let sc = sc.clone();
    let out = match tokio::task::spawn_blocking(move || crate::db::gdlr_media_json(&sc.db, &site_url(&sc))).await {
        Ok(out) => out,
        Err(e) => {
            eprintln!("gdlr media_list: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response();
        }
    };
    match out {
        Ok(j) => ([(header::CONTENT_TYPE, "application/json")], j).into_response(),
        Err(e) => {
            eprintln!("gdlr media_list: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
        }
    }
}
