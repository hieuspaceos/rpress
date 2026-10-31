//! Presto Player — `[presto_player id=N]` shortcode + `pp_video_block`
//! storage-CPT harvest.
//!
//! On WP, each video is a `pp_video_block` post whose post_content holds
//! a Gutenberg comment (`<!-- wp:presto-player/youtube {"src":…,
//! "video_id":…} /-->`). The CPT itself is plugin bookkeeping — nothing
//! routable — so import still skips it, but `post_seen` harvests the
//! video_id into `settings` (`plugin:presto_player:video:<id>`) before
//! the row is dropped. `expand_shortcodes` then resolves
//! `[presto_player id=N]` to a YouTube iframe (the only provider
//! observed in dumps; non-YouTube srcs degrade to a bare link, same as
//! `[embed]`).

use super::{Backend, ImportHooks, KindDef, PluginAdapter, PluginDescriptor};
use anyhow::Result;
use rusqlite::Connection;

pub(crate) struct PrestoPlayer;
pub(crate) static ADAPTER: PrestoPlayer = PrestoPlayer;

/// Settings key for one harvested video: wp post_id → YouTube id.
pub(crate) fn video_key(post_id: i64) -> String {
    format!("plugin:presto_player:video:{post_id}")
}

/// First `wp:presto-player/*` block payload's `video_id` inside one
/// `pp_video_block` post_content. The block JSON sits inside a Gutenberg
/// comment — a string scan beats pulling a JSON parser for one key.
fn harvest_video_id(content: &str) -> Option<String> {
    let start = content.find("wp:presto-player/")?;
    let seg = &content[start..content.len().min(start + 2048)];
    let key = seg.find("\"video_id\":\"")? + "\"video_id\":\"".len();
    let end = seg[key..].find('"')? + key;
    let id = &seg[key..end];
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return None;
    }
    Some(id.to_string())
}

fn seen(c: &Connection, id: i64, post_type: &str, content: &str) -> Result<()> {
    if post_type == "pp_video_block" {
        if let Some(vid) = harvest_video_id(content) {
            // OR IGNORE: a duplicate post_id (WP autosave artifact) keeps
            // the first harvested id — deterministic.
            c.execute(
                crate::sql!("settings__insert_or_ignore"),
                rusqlite::params![video_key(id), vid],
            )?;
        }
    }
    Ok(())
}

fn post_noop(_c: &Connection, _id: i64) -> Result<()> {
    Ok(())
}
fn after_noop(_c: &Connection) -> Result<()> {
    Ok(())
}
fn claim(_t: &str) -> Option<&'static str> {
    None
}

static HOOKS: ImportHooks = ImportHooks {
    claim_type: claim,
    option_prefixes: &["presto_", "presto_player", "presto-player"],
    post_seen: seen,
    post_imported: post_noop,
    after_import: after_noop,
};

impl PluginAdapter for PrestoPlayer {
    fn slug(&self) -> &'static str {
        "presto-player"
    }
    fn kinds(&self) -> &'static [KindDef] {
        &[]
    }
    fn backend(&self) -> Backend {
        Backend::Rust
    }
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            name: "Presto Player",
            blurb: "Video blocks — [presto_player] rendered as a YouTube embed",
        }
    }
    fn ignore_types(&self) -> &'static [&'static str] {
        // The video records are harvested via `post_seen`, not imported —
        // keep the CPT out of detect's UNCLAIMED list.
        &["pp_video_block"]
    }
    fn rendered_shortcodes(&self) -> &'static [&'static str] {
        &["presto_player"]
    }
    fn import_hooks(&self) -> Option<&'static ImportHooks> {
        Some(&HOOKS)
    }
}

#[cfg(test)]
mod tests {
    use super::harvest_video_id;

    #[test]
    fn harvests_youtube_block() {
        let c = r#"<!-- wp:presto-player/reusable-edit -->
<div class="x"><!-- wp:presto-player/youtube {"id":6,"src":"https://youtu.be/aL7KuNPAfe0","preset":1,"video_id":"aL7KuNPAfe0"} /--></div>
<!-- /wp:presto-player/reusable-edit -->"#;
        assert_eq!(harvest_video_id(c).as_deref(), Some("aL7KuNPAfe0"));
    }

    #[test]
    fn ignores_non_presto() {
        assert_eq!(harvest_video_id("<p>hello</p>"), None);
        assert_eq!(harvest_video_id(r#"wp:presto-player/youtube {"video_id":""}"#), None);
    }
}
