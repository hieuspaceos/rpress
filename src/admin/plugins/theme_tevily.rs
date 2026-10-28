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
    body_suffix: "wp-theme-tevily wp-child-theme-tevily_child tevily-body-loading elementor-default",
    page_title_wrap: "",
    preferred_locations: &["header-navigation", "main-menu", "main_navigation", "primary"],
};

use anyhow::Result;
use std::path::Path;

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
    /// Seed the tevily body-class suffix so `body_class()` stops falling
    /// back to traveltour's `wp-theme-traveltour` tail. Idempotent
    /// (`INSERT OR REPLACE`); elementor kit ids vary per page so they are
    /// omitted here (render adds page-id-<id> itself).
    fn migrate(&self, db: &Path) -> Result<()> {
        let c = crate::db::conn(db)?;
        let stylesheet: String = c
            .query_row("SELECT value FROM settings WHERE key='stylesheet'", [], |r| r.get(0))
            .unwrap_or_default();
        if !(THEME.matches_stylesheet)(&stylesheet) {
            return Ok(());
        }
        c.execute(
            "INSERT OR REPLACE INTO settings(key, value) VALUES('site_body_suffix', ?1)",
            ["wp-theme-tevily wp-child-theme-tevily_child tevily-body-loading elementor-default"],
        )?;
        Ok(())
    }
}

pub(crate) static ADAPTER: ThemeTevily = ThemeTevily;
