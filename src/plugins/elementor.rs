//! Elementor — `Backend::PhpBridge` (phase-00 spike).
//! Real plugin in `www/wp-content/plugins/elementor`. rpress only
//! mounts `/admin/plugins/elementor/{*}` onto the sidecar.

use super::{Backend, PluginAdapter};
use crate::admin::bridge::pull::WriteSpec;

pub(crate) struct Elementor;
pub(crate) static ADAPTER: Elementor = Elementor;

// Spike: no write_specs. `_elementor%` is illegal today (`_` is LIKE
// wildcard in pull::WriteSpec::validate) and specs are UNION'd on every
// bridge pull — a bad Elementor spec would 502 GoodLayers saves.
static WRITE_SPECS: &[WriteSpec] = &[];

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
