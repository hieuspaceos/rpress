//! GoodLayers media endpoint used by the wp.media shim.

use super::*;
/// GET /api/v1/gdlr/media — id/path/alt list for the upload field picker.
pub(crate) async fn media_list(State(app): State<App>, headers: HeaderMap) -> Response {
    let Ok((_, sc)) = need_auth(&app, &headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let out = crate::db::gdlr_media_json(&sc.db, &site_url(sc));
    match out {
        Ok(j) => ([(header::CONTENT_TYPE, "application/json")], j).into_response(),
        Err(e) => {
            eprintln!("gdlr media_list: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
        }
    }
}
