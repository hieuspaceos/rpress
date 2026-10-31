# Phase 01 — EditorMode

- Add `EditorMode { None, Embedded, Canvas { label, rest } }` on `PluginAdapter` (default `None`).
- GoodLayers returns `Embedded`; `posts.rs` iterates adapters instead of `adapter_for_slug("goodlayers")`.
- Canvas: button only, no metabox.
- Verify: aucoeur post-edit still shows Page Builder metabox; `cargo test`.
