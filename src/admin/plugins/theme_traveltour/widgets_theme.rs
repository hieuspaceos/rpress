//! traveltour theme widget data — the sidebar regions and seed payloads
//! that used to be compiled into `widgets.rs` as the core `SIDEBARS` /
//! `SEED_WIDGETS` tables. Moved verbatim (phase 04, theme-as-data); the
//! adapter returns them through `PluginAdapter::sidebars`/`seeds` so core
//! merges them via `all_sidebars()`/`seed_widgets()` unchanged.
//!
//! Seed provenance (verbatim extraction of the pre-registry sources):
//!   sidebar-post-fr ← the old templates/sidebar_post.html FR capture
//!                   (search-4, media_image-4; wpcategorieswidget-9 is now a
//!                   LIVE categories widget — the frozen capture became the
//!                   `category` wp-id map + tree)
//!   sidebar-post-es ← phase-3 hand-translation (NO ES capture exists —
//!                   new content, not byte-parity)
//!   footer-N-{fr,es} ← the traveltour-footer-column blocks in
//!                   templates/base_fr.html / base_es.html
//!   sidebar-archive-{fr,es} ← the WIDGETS const (FR, verbatim) + ES mirror

use crate::widgets::{Seed, Sidebar};

const POST_AREA_BEFORE: &str = r#"<div class=" traveltour-sidebar-right traveltour-column-15 traveltour-line-height traveltour-line-height" ><div class="traveltour-sidebar-area traveltour-item-pdlr" >"#;
const POST_AREA_AFTER: &str = r#"</div></div></div></div>"#;

/// Theme sidebar regions (wp `register_sidebar` equivalents): the blog
/// sidebar per language, the tour-archive sidebar per language, and the
/// four footer columns per language.
pub(crate) const SIDEBARS: &[Sidebar] = &[
    Sidebar { id: "sidebar-post-fr", name: "Blog sidebar (FR)", lang: "fr",
        before: POST_AREA_BEFORE,
        after: POST_AREA_AFTER },
    Sidebar { id: "sidebar-post-es", name: "Blog sidebar (ES)", lang: "es",
        before: POST_AREA_BEFORE,
        after: POST_AREA_AFTER },
    Sidebar { id: "sidebar-archive-fr", name: "Tour archive sidebar (FR)", lang: "fr",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "sidebar-archive-es", name: "Tour archive sidebar (ES)", lang: "es",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "footer-1-fr", name: "Footer column 1 (FR)", lang: "fr",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "footer-2-fr", name: "Footer column 2 (FR)", lang: "fr",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "footer-3-fr", name: "Footer column 3 (FR)", lang: "fr",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "footer-4-fr", name: "Footer column 4 (FR)", lang: "fr",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "footer-1-es", name: "Footer column 1 (ES)", lang: "es",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "footer-2-es", name: "Footer column 2 (ES)", lang: "es",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "footer-3-es", name: "Footer column 3 (ES)", lang: "es",
        before: r#""#,
        after: r#""# },
    Sidebar { id: "footer-4-es", name: "Footer column 4 (ES)", lang: "es",
        before: r#""#,
        after: r#""# },
];

/// Theme seed payloads — moved verbatim from `src/seed/`.
mod seed {
    pub const SIDEBAR_POST_FR_SEARCH_4: &str = include_str!("seed/sidebar-post-fr-search-4.json");
    pub const SIDEBAR_POST_FR_WPCATEGORIESWIDGET_9: &str = include_str!("seed/sidebar-post-fr-wpcategorieswidget-9.json");
    pub const SIDEBAR_POST_FR_MEDIA_IMAGE_4: &str = include_str!("seed/sidebar-post-fr-media_image-4.json");
    pub const SIDEBAR_POST_ES_SEARCH_5: &str = include_str!("seed/sidebar-post-es-search-5.json");
    pub const SIDEBAR_POST_ES_WPCATEGORIESWIDGET_10: &str = include_str!("seed/sidebar-post-es-wpcategorieswidget-10.json");
    pub const SIDEBAR_POST_ES_MEDIA_IMAGE_5: &str = include_str!("seed/sidebar-post-es-media_image-5.json");
    pub const FOOTER_1_FR_TEXT_1: &str = include_str!("seed/footer-1-fr-text-1.json");
    pub const FOOTER_2_FR_CUSTOM_HTML_3: &str = include_str!("seed/footer-2-fr-custom_html-3.json");
    pub const FOOTER_2_FR_TEXT_17: &str = include_str!("seed/footer-2-fr-text-17.json");
    pub const FOOTER_3_FR_CUSTOM_HTML_5: &str = include_str!("seed/footer-3-fr-custom_html-5.json");
    pub const FOOTER_3_FR_TEXT_15: &str = include_str!("seed/footer-3-fr-text-15.json");
    pub const FOOTER_4_FR_CUSTOM_HTML_6: &str = include_str!("seed/footer-4-fr-custom_html-6.json");
    pub const FOOTER_1_ES_TEXT_20: &str = include_str!("seed/footer-1-es-text-20.json");
    pub const FOOTER_2_ES_CUSTOM_HTML_9: &str = include_str!("seed/footer-2-es-custom_html-9.json");
    pub const FOOTER_2_ES_TEXT_21: &str = include_str!("seed/footer-2-es-text-21.json");
    pub const FOOTER_3_ES_CUSTOM_HTML_10: &str = include_str!("seed/footer-3-es-custom_html-10.json");
    pub const FOOTER_3_ES_TEXT_22: &str = include_str!("seed/footer-3-es-text-22.json");
    pub const FOOTER_4_ES_CUSTOM_HTML_11: &str = include_str!("seed/footer-4-es-custom_html-11.json");
    pub const SIDEBAR_ARCHIVE_FR_WPCATEGORIESWIDGET_5: &str = include_str!("seed/sidebar-archive-fr-wpcategorieswidget-5.json");
    pub const SIDEBAR_ARCHIVE_FR_WPCATEGORIESWIDGET_7: &str = include_str!("seed/sidebar-archive-fr-wpcategorieswidget-7.json");
    pub const SIDEBAR_ARCHIVE_FR_WPCATEGORIESWIDGET_8: &str = include_str!("seed/sidebar-archive-fr-wpcategorieswidget-8.json");
    pub const SIDEBAR_ARCHIVE_FR_WPCATEGORIESWIDGET_6: &str = include_str!("seed/sidebar-archive-fr-wpcategorieswidget-6.json");
    pub const SIDEBAR_ARCHIVE_ES_WPCATEGORIESWIDGET_11: &str = include_str!("seed/sidebar-archive-es-wpcategorieswidget-11.json");
    pub const SIDEBAR_ARCHIVE_ES_WPCATEGORIESWIDGET_12: &str = include_str!("seed/sidebar-archive-es-wpcategorieswidget-12.json");
    pub const SIDEBAR_ARCHIVE_ES_WPCATEGORIESWIDGET_13: &str = include_str!("seed/sidebar-archive-es-wpcategorieswidget-13.json");
    pub const SIDEBAR_ARCHIVE_ES_WPCATEGORIESWIDGET_14: &str = include_str!("seed/sidebar-archive-es-wpcategorieswidget-14.json");
}

/// Theme seed widget rows — verbatim the old `SEED_WIDGETS` table.
/// `PluginAdapter::seeds` returns these; `seed_widgets` chains them after
/// the (now empty) core table, preserving per-sidebar `ord` counters.
pub(crate) static SEEDS: &[Seed] = &[
    Seed { sidebar: "sidebar-post-fr", id: "search-4", typ: "search", title: "",
        settings: seed::SIDEBAR_POST_FR_SEARCH_4 },
    Seed { sidebar: "sidebar-post-fr", id: "wpcategorieswidget-9", typ: "categories", title: "Catégories",
        settings: seed::SIDEBAR_POST_FR_WPCATEGORIESWIDGET_9 },
    Seed { sidebar: "sidebar-post-fr", id: "media_image-4", typ: "media_image", title: "",
        settings: seed::SIDEBAR_POST_FR_MEDIA_IMAGE_4 },
    Seed { sidebar: "sidebar-post-es", id: "search-5", typ: "search", title: "",
        settings: seed::SIDEBAR_POST_ES_SEARCH_5 },
    Seed { sidebar: "sidebar-post-es", id: "wpcategorieswidget-10", typ: "categories", title: "Categorías",
        settings: seed::SIDEBAR_POST_ES_WPCATEGORIESWIDGET_10 },
    Seed { sidebar: "sidebar-post-es", id: "media_image-5", typ: "media_image", title: "",
        settings: seed::SIDEBAR_POST_ES_MEDIA_IMAGE_5 },
    Seed { sidebar: "footer-1-fr", id: "text-1", typ: "text", title: "",
        settings: seed::FOOTER_1_FR_TEXT_1 },
    Seed { sidebar: "footer-2-fr", id: "custom_html-3", typ: "custom_html", title: "",
        settings: seed::FOOTER_2_FR_CUSTOM_HTML_3 },
    Seed { sidebar: "footer-2-fr", id: "text-17", typ: "text", title: "",
        settings: seed::FOOTER_2_FR_TEXT_17 },
    Seed { sidebar: "footer-3-fr", id: "custom_html-5", typ: "custom_html", title: "",
        settings: seed::FOOTER_3_FR_CUSTOM_HTML_5 },
    Seed { sidebar: "footer-3-fr", id: "text-15", typ: "text", title: "",
        settings: seed::FOOTER_3_FR_TEXT_15 },
    Seed { sidebar: "footer-4-fr", id: "custom_html-6", typ: "custom_html", title: "",
        settings: seed::FOOTER_4_FR_CUSTOM_HTML_6 },
    Seed { sidebar: "footer-1-es", id: "text-20", typ: "text", title: "",
        settings: seed::FOOTER_1_ES_TEXT_20 },
    Seed { sidebar: "footer-2-es", id: "custom_html-9", typ: "custom_html", title: "",
        settings: seed::FOOTER_2_ES_CUSTOM_HTML_9 },
    Seed { sidebar: "footer-2-es", id: "text-21", typ: "text", title: "",
        settings: seed::FOOTER_2_ES_TEXT_21 },
    Seed { sidebar: "footer-3-es", id: "custom_html-10", typ: "custom_html", title: "",
        settings: seed::FOOTER_3_ES_CUSTOM_HTML_10 },
    Seed { sidebar: "footer-3-es", id: "text-22", typ: "text", title: "",
        settings: seed::FOOTER_3_ES_TEXT_22 },
    Seed { sidebar: "footer-4-es", id: "custom_html-11", typ: "custom_html", title: "",
        settings: seed::FOOTER_4_ES_CUSTOM_HTML_11 },
    Seed { sidebar: "sidebar-archive-fr", id: "wpcategorieswidget-5", typ: "categories", title: "Voyage par region",
        settings: seed::SIDEBAR_ARCHIVE_FR_WPCATEGORIESWIDGET_5 },
    Seed { sidebar: "sidebar-archive-fr", id: "wpcategorieswidget-7", typ: "categories", title: "Voyage par profil",
        settings: seed::SIDEBAR_ARCHIVE_FR_WPCATEGORIESWIDGET_7 },
    Seed { sidebar: "sidebar-archive-fr", id: "wpcategorieswidget-8", typ: "categories", title: "Voyage par durée",
        settings: seed::SIDEBAR_ARCHIVE_FR_WPCATEGORIESWIDGET_8 },
    Seed { sidebar: "sidebar-archive-fr", id: "wpcategorieswidget-6", typ: "categories", title: "Voyage par styles",
        settings: seed::SIDEBAR_ARCHIVE_FR_WPCATEGORIESWIDGET_6 },
    Seed { sidebar: "sidebar-archive-es", id: "wpcategorieswidget-11", typ: "categories", title: "Viaje por región",
        settings: seed::SIDEBAR_ARCHIVE_ES_WPCATEGORIESWIDGET_11 },
    Seed { sidebar: "sidebar-archive-es", id: "wpcategorieswidget-12", typ: "categories", title: "Viaje por perfil",
        settings: seed::SIDEBAR_ARCHIVE_ES_WPCATEGORIESWIDGET_12 },
    Seed { sidebar: "sidebar-archive-es", id: "wpcategorieswidget-13", typ: "categories", title: "Viaje por duración",
        settings: seed::SIDEBAR_ARCHIVE_ES_WPCATEGORIESWIDGET_13 },
    Seed { sidebar: "sidebar-archive-es", id: "wpcategorieswidget-14", typ: "categories", title: "Viaje por estilos",
        settings: seed::SIDEBAR_ARCHIVE_ES_WPCATEGORIESWIDGET_14 },
];
