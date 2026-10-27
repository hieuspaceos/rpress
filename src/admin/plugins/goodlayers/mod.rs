//! GoodLayers (`goodlayers-core`) adapter — the `Backend::PhpBridge`
//! plugin (phase-07). Real WP + real plugin run inside the bridge
//! runtime; rpress only:
//!
//!  * emits the builder DOM skeleton + JS vars on the post-edit screen
//!    (`editor_boot`/`editor_assets`, reusing the gdlr_pb emitters —
//!    they're the *JS contract*, not business logic: heavy parts —
//!    option renderers, ajax dispatch — die with the port once the
//!    parity checklist in `phase-07-gdlr-bridge.md` is signed);
//!  * declares the pull-write whitelist: `post_meta` `gdlr-core-%`,
//!    `settings` `gdlr-core-%` → `plugin:goodlayers:` namespace, and
//!    the `gdlr-core-page-builder` meta → `posts.body_json` derived
//!    capability (same write the Rust port made — render authority
//!    stays `body_json`).
//!
//! `/admin/api/gdlr/ajax` 307s to this screen for every site — the
//! goodlayers bridge is always-on (no non-bridge PB path remains).

use super::{Backend, PluginAdapter};
use crate::admin::bridge::pull::{PullTable, WriteSpec};
use crate::admin::SiteCtx;
use rusqlite::Connection;

pub(crate) mod boot;

pub(crate) struct GoodLayers;
pub(crate) static ADAPTER: GoodLayers = GoodLayers;

/// New ajax endpoint — the proxied screen. Rewritten into boot output
/// only when `PLUGINS_PHP` includes this slug (otherwise the old
/// `/admin/api/gdlr/ajax` dispatch stays live and pages keep working).
pub(crate) const AJAX_URL: &str = "/admin/plugins/goodlayers/ajax";
const LEGACY_AJAX: &str = "/admin/api/gdlr/ajax";

static WRITE_SPECS: &[WriteSpec] = &[
    WriteSpec { table: PullTable::PostMeta, key_column: "meta_key", key_prefix: "gdlr-core-%", key_map: None },
    WriteSpec {
        table: PullTable::Settings,
        key_column: "key",
        key_prefix: "gdlr-core-%",
        key_map: Some("plugin:goodlayers:"),
    },
    WriteSpec { table: PullTable::BodyJson, key_column: "meta_key", key_prefix: "gdlr-core-page-builder", key_map: None },
];

impl PluginAdapter for GoodLayers {
    fn slug(&self) -> &'static str {
        "goodlayers"
    }
    fn kinds(&self) -> &'static [super::KindDef] {
        &[]
    }
    fn backend(&self) -> Backend {
        Backend::PhpBridge
    }
    fn write_specs(&self) -> &'static [WriteSpec] {
        WRITE_SPECS
    }
    /// Bare legacy keys imported from wp_options — folded into
    /// `plugin:goodlayers:` at boot by `migrate_all`.
    fn legacy_option_keys(&self) -> &'static [&'static str] {
        &[
            "gdlr-core-skins",
            "gdlr-core-skins-value",
            "gdlr-core-pb-template-page",
            "gdlr-core-pb-templates",
            "gdlr-core-pb-custom-template",
            "gdlr-core-pb-sync-template",
        ]
    }
    /// `ajax` (and anything under it) reaches WP's admin-ajax — the 13
    /// `gdlr_core_*` actions are dispatched by the real plugin inside
    /// the bridge. Other paths pass through verbatim (asset/screen
    /// routes the bridge WP owns).
    fn bridge_path(&self, rest: &str) -> String {
        if rest == "ajax" || rest.starts_with("ajax/") {
            "/wp-admin/admin-ajax.php".to_string()
        } else {
            format!("/{rest}")
        }
    }
    fn wp_plugin_file(&self) -> &'static str {
        "goodlayers-core/goodlayers-core.php"
    }
    /// The builder boot block. When the bridge is enabled for this
    /// site the ajax endpoints inside the emitted markup are rewritten
    /// to `/admin/plugins/goodlayers/ajax`; otherwise the legacy route
    /// keeps working (parity fallback — the port dies only after the
    /// phase-07 checklist passes on a real session).
    fn editor_boot(
        &self,
        conn: &Connection,
        sc: &SiteCtx,
        post_id: i64,
        body_json: &str,
        csrf: &str,
    ) -> Option<String> {
        let mut html = boot::boot(conn, sc, post_id, body_json, csrf);
        if crate::admin::bridge::opt_in(&sc.plugins_php, self.slug()) {
            html = html.replace(LEGACY_AJAX, AJAX_URL);
        }
        Some(html)
    }
    fn editor_assets(&self, sc: &SiteCtx) -> Option<String> {
        let tags = boot::asset_tags();
        if crate::admin::bridge::opt_in(&sc.plugins_php, self.slug()) {
            Some(tags.replace(LEGACY_AJAX, AJAX_URL))
        } else {
            Some(tags)
        }
    }
}
