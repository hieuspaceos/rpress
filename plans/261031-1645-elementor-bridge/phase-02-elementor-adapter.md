# Phase 02 — Adapter

- `src/plugins/elementor/mod.rs` like `goodlayers/mod.rs`.
- `write_specs`: `_elementor%`, `elementor_%` → `plugin:elementor:`.
- `bridge_path` for `ajax`, `wp-admin/post.php`, frontend `index.php`.
- mu-plugin: if `$_POST['action']==='elementor_ajax'` mint `wp_create_nonce('elementor_ajax')` into `_nonce`.
- Button on post-edit iff `_elementor_data` && `opt_in`. Do not touch GDLR `body_is_gdlr` branch.
