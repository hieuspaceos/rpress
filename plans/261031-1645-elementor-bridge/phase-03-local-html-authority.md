# Phase 03 — Public HTML

`save_plain_text` is **not** the page. After `save_builder` pull:

1. Copy `_elementor%` meta (existing WriteSpec).
2. GET bridge `/?p={wp_id}` → store HTML.
3. `generate.rs`: skip origin harvest when that HTML exists.
4. Regen `cache/<slug>/index.html`.

CSS file `uploads/elementor/css/post-{id}.css` is www — no extra pull.
