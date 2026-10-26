//! GoodLayers media endpoint used by the wp.media shim.

use super::*;
use serde_json::Value;
/// GET /api/v1/gdlr/media — id/path/alt list for the upload field picker.
pub(crate) async fn media_list(State(app): State<App>, headers: HeaderMap) -> Response {
    let Ok((_, sc)) = need_auth(&app, &headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let out = (|| -> anyhow::Result<String> {
        let conn = crate::db::conn(&sc.db)?;
        let mut stmt = conn.prepare("SELECT id, path, COALESCE(alt,''), COALESCE(mime,'') FROM media ORDER BY id DESC")?;
        let rows = stmt.query_map([], |r| {
            Ok(serde_json::json!({"id": r.get::<_, i64>(0)?, "path": r.get::<_, String>(1)?, "alt": r.get::<_, String>(2)?, "mime": r.get::<_, String>(3)?}))
        })?;
        let site = site_url(sc).trim_end_matches('/').to_string();
        let v: Vec<Value> = rows.flatten().map(|mut m| {
            if let Some(p) = m.get("path").and_then(|p| p.as_str()).map(|x| x.to_string()) {
                m.as_object_mut().unwrap().insert("url".into(), serde_json::json!(format!("{site}/wp-content/uploads/{p}")));
            }
            m
        }).collect();
        Ok(serde_json::to_string(&v)?)
    })();
    match out {
        Ok(j) => ([(header::CONTENT_TYPE, "application/json")], j).into_response(),
        Err(e) => {
            eprintln!("gdlr media_list: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
        }
    }
}
