//! Read More Without Refresh — `[read]…[/read]` shortcode.
//!
//! Static parity: the WP plugin hides content behind a JS toggle; on a
//! generated page there's no script, so `expand_shortcodes` renders the
//! enclosed content inline (expanded). Unclosed `[read]`/`[read more]`
//! tags are stripped — WP prints the raw tag text, but the stripped
//! variant keeps dumps readable when the closer was lost upstream.

use super::{Backend, KindDef, PluginAdapter, PluginDescriptor};

pub(crate) struct ReadMore;
pub(crate) static ADAPTER: ReadMore = ReadMore;

impl PluginAdapter for ReadMore {
    fn slug(&self) -> &'static str {
        "read-more-without-refresh"
    }
    fn kinds(&self) -> &'static [KindDef] {
        &[]
    }
    fn backend(&self) -> Backend {
        Backend::Rust
    }
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            name: "Read More Without Refresh",
            blurb: "[read] toggles — rendered expanded (no JS on static pages)",
        }
    }
    fn settings_prefixes(&self) -> &'static [&'static str] {
        &["rmwr_", "rl_"]
    }
    fn rendered_shortcodes(&self) -> &'static [&'static str] {
        &["read"]
    }
}
