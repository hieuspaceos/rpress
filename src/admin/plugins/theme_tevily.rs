//! `tevily` theme adapter — stylesheet stub so `theme_for_stylesheet`
//! resolves `settings.stylesheet='tevily'` (and `tevily-child*`) sites.
//! Real tevily templates land in phase 06; until then `template_dir`
//! points at an empty dir and `load_env`/`template_name` fall through to
//! the shared root (keeps current aucoeur-chrome fallback for tevily
//! sites — no worse than today's state).
//!
//! Register via `ADAPTERS`; `migrate` is a no-op (no seeds, no schemas
//! — shared templates serve until real markup exists).

use crate::admin::plugins::{Backend, PluginAdapter, ThemeDef};

pub(crate) struct ThemeTevily;

static THEME: ThemeDef = ThemeDef {
    matches_stylesheet: |s| {
        // WP child-theme dir conventions seen in the wild: both dash and
        // underscore separators (`tevily-child`, `tevily_child`).
        s == "tevily"
            || s == "tevily-child"
            || s == "tevily_child"
            || s.starts_with("tevily-child-")
            || s.starts_with("tevily_child-")
            || s.starts_with("tevily_child_")
    },
    template_dir: "tevily",
    body_suffix: "",
    page_title_wrap: "",
    preferred_locations: &["header-navigation", "main-menu", "main_navigation", "primary"],
};

impl PluginAdapter for ThemeTevily {
    fn slug(&self) -> &'static str {
        "theme-tevily"
    }
    fn kinds(&self) -> &'static [super::KindDef] {
        &[]
    }
    fn backend(&self) -> Backend {
        Backend::Data
    }
    fn theme(&self) -> Option<&'static ThemeDef> {
        Some(&THEME)
    }
}

pub(crate) static ADAPTER: ThemeTevily = ThemeTevily;
