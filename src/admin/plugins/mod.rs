//! Per-kind editor plugins. A plugin owns everything about editing one
//! post `kind`: its metabox HTML, its admin assets, and its save path.
//! Register by adding `&PLUGIN` to `PLUGINS`; commenting the line out is
//! the off switch — the core never names plugins.
//!
//! Contract the core guarantees: `SiteCtx`, `PostForm.extra` (serde-flatten
//! capture of plugin form fields), `post_meta` helpers in `db::posts`,
//! `php_serialize`, the `/admin/wp/{*path}` asset route, WP form chrome
//! (`#post` posts to `/admin/save`, `#publish` is the submit button).

use crate::db::posts::PostForm;
use crate::admin::{Sess, SiteCtx};
use anyhow::Result;
use rusqlite::Connection;

pub(crate) mod tourmaster;

pub(crate) struct EditorPlugin {
    /// `posts.kind` this plugin edits, e.g. "tour".
    pub(crate) kind: &'static str,
    /// Hydrate `PostForm.extra` on load (e.g. unserialize meta blobs into
    /// the JSON the metabox expects).
    pub(crate) load: fn(&Connection, i64, &mut PostForm) -> Result<()>,
    /// Panel HTML emitted inside `post-body-content`, after the editor.
    pub(crate) metabox: fn(&SiteCtx, &Sess, &PostForm) -> String,
    /// `<link>`/`<script>` tags appended to the edit page.
    pub(crate) assets: fn() -> &'static str,
    /// Runs inside `crud::save_post`'s connection after the posts row
    /// upsert; plugin reads its fields from `f.extra`.
    pub(crate) save: fn(&Connection, i64, &PostForm) -> Result<()>,
}

pub(crate) static PLUGINS: &[&EditorPlugin] = &[&tourmaster::PLUGIN];

pub(crate) fn find(kind: &str) -> Option<&'static EditorPlugin> {
    PLUGINS.iter().copied().find(|p| p.kind == kind)
}
