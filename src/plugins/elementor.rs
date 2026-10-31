//! Elementor — `Backend::PhpBridge` (phase-00 spike).
//! Real plugin in `www/wp-content/plugins/elementor`. rpress only
//! mounts `/admin/plugins/elementor/{*}` onto the sidecar.

use super::{Backend, PluginAdapter};
use crate::admin::bridge::pull::{PullTable, WriteSpec};

pub(crate) struct Elementor;
pub(crate) static ADAPTER: Elementor = Elementor;

// Elementor saves land in wp.db as `_elementor*` post meta plus the
// compiled markup in `wp_posts.post_content`. Pull both: meta verbatim
// into `post_meta`, post_content/post_status onto posts.body_html/
// status (body_json is rebuilt only when it is the generic import
// text-box — the renderer can't consume Elementor's own JSON).
static WRITE_SPECS: &[WriteSpec] = &[
    WriteSpec { table: PullTable::PostMeta, key_column: "meta_key", key_prefix: "_elementor", key_map: None },
    WriteSpec { table: PullTable::PostFields, key_column: "meta_key", key_prefix: "", key_map: None },
];

impl PluginAdapter for Elementor {
    fn slug(&self) -> &'static str {
        "elementor"
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
    fn bridge_path(&self, rest: &str) -> String {
        if rest == "ajax" || rest.starts_with("ajax/") {
            "/wp-admin/admin-ajax.php".to_string()
        } else if rest.is_empty() || rest == "index.php" {
            // `/index.php?...` makes WP canonical-redirect to `/?...` which
            // we rewrite back onto this mount → loop. php -S serves `/` as index.php.
            "/".to_string()
        } else {
            format!("/{rest}")
        }
    }
    fn wp_plugin_file(&self) -> &'static str {
        "elementor/elementor.php"
    }
}
