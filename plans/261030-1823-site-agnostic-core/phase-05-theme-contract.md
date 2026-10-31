---
title: "Phase 5: Theme Contract — Hybrid TOML + Rust"
status: done
priority: P0
---

# Phase 5: Theme Contract — Hybrid TOML + Rust

## Overview

Make "new theme" a data operation, not a core-code change. User decision:
**hybrid** — extend `ThemeDef` with a declarative TOML theme spec (mirroring
plugin specs) covering URL shapes, taxonomy→lang, and class prefixes; keep
the Rust `PluginAdapter::theme()` as escape hatch for themes with code-level
quirks. A neutral default theme ships in core.

## File Ownership

- `src/plugins/mod.rs` — extend `ThemeDef` (new fields), `theme_for_stylesheet` resolution incl. TOML-generated defs (depends on phase 6's spec infra? — no: theme specs get their own `themes/specs/*.toml` loader or reuse `plugins/specs` with a `[theme]` section; **chosen: separate `themes/specs/*.toml` embedded by build.rs, same codegen pattern as plugin spec**)
- `src/plugins/theme_traveltour/` — becomes the reference Rust-backed theme adapter; its URL/class logic moves behind `ThemeDef` methods
- `src/render.rs` — replace `crate::plugins::theme_traveltour::cat_path/tag_base` direct calls (:126,265,311-313) + `traveltour-content-container`/`sidebar-wrap` literals (:555-596) + template fallback literal (:992)
- `src/generate.rs` — `region-es|perfil|duracion|estilo` → `"es"` hardcode (:362-382) → `ThemeDef::tax_lang(taxonomy) -> Option<&str>`
- `src/admin/mod.rs` — `tax_label` literals (:1215-1221), `traveltour_color` option read (:1122), `chrome_start` theme assumptions
- `src/admin/widgets.rs` — `traveltour-widget*` class literals (:728,736,886,893)
- `src/admin/rankmath.rs` — primary-taxonomy name literals (:21-23)
- `src/render_post.rs` — any theme literals found during ledger pass
- `themes/specs/*.toml` (new), `build.rs` (embed), `templates/theme/` (neutral default dir)

## Design: `ThemeDef` v2

Extend the existing struct (mod.rs:497-510) — add the fields the audit found
hardcoded:

```rust
pub(crate) struct ThemeDef {
    pub matches_stylesheet: fn(&str) -> bool,
    pub template_dir: &'static str,
    pub body_suffix: &'static str,
    pub page_title_wrap: &'static str,
    pub preferred_locations: &'static [&'static str],
    // NEW — all the literals being evicted:
    pub cat_path: fn(&str /*cat slug*/) -> String,   // was theme_traveltour::cat_path
    pub tag_base: &'static str,                      // was theme_traveltour::tag_base
    pub tax_lang: fn(&str /*taxonomy*/) -> Option<&'static str>,  // was generate.rs literal map
    pub tax_labels: &'static [(&'static str, &'static str)],      // was admin tax_label
    pub widget_class: &'static str,                  // "traveltour-widget" prefix
    pub sidebar_wrap_class: &'static str,            // "traveltour-content-container ..."
    pub body_class: &'static str,                    // "traveltour-body" live-edit
    pub settings_key_prefix: &'static str,           // "traveltour_" option reads
}
```

TOML spec (static-friendly subset — `cat_path`/`tax_lang` become data):

```toml
# themes/specs/minimal.toml
stylesheet_patterns = ["minimal*"]
template_dir = "minimal"
body_suffix = "minimal-body"
page_title_wrap = ""
preferred_locations = ["primary"]
tag_base = "tag"
cat_path_prefix = "category"          # → cat_path closure generated
widget_class = "minimal-widget"
sidebar_wrap_class = "minimal-content-container minimal-sidebar-wrap"
body_class = "minimal-body"
settings_key_prefix = "minimal_"

[[tax_lang]]
taxonomy = "region-es"
lang = "es"

[[tax_labels]]
taxonomy = "region-es"
label = "Region"
```

Resolution order: `theme_for_stylesheet(stylesheet)` checks Rust adapters
first, then TOML-generated defs (Rust escape hatch wins, same as plugin spec
priority). A **neutral `default` theme** TOML ships in core and is the
fallback when nothing matches — template dir `templates/theme/default` with
generic semantic markup (no site brand).

## Requirements

- [ ] Zero `theme_traveltour` imports outside `plugins/` — all call sites go through `theme_for_stylesheet(settings.stylesheet)` (fall back to default theme when `None`).
- [ ] All literals in the audit ledger moved behind `ThemeDef` or TOML data: render.rs:126/265/311-313/555-596/992; generate.rs:362-382; mod.rs:1215-1221+1122; widgets.rs:728-893; rankmath.rs:21-23; goodlayers/boot.rs:1522,1573 (live-edit classes — these belong to the goodlayers bridge adapter's theme interaction; route via ThemeDef fields).
- [ ] TOML theme spec loader in build.rs + spec-style codegen (reuse `spec.rs` patterns: `deny_unknown_fields`, fail-loud parse, collision panic if a TOML theme collides with a Rust theme's stylesheet patterns).
- [ ] Neutral `default` theme: TOML + `templates/theme/default/` minimal templates (base + post + page) so an unmatched stylesheet still renders legibly.
- [ ] `theme_template_dir` fallback literal `templates/theme/traveltour` (render.rs:992) → resolved theme's `template_dir`.
- [ ] Widget chrome emits `{theme.widget_class}*`; sidebar wrapper emits `{theme.sidebar_wrap_class}`; admin `traveltour_color` read → `{theme.settings_key_prefix}color` (and keep a settings-fallback so old DBs still work — read `plugin:`-style: prefer prefixed key, fall back to literal legacy key).
- [ ] Proof-of-concept second theme: convert `tevily` (existing reference site theme) to a TOML spec + templates — no Rust adapter — and render a tevily site without traveltour leakage.

## Implementation Steps

1. Literal ledger: grep the full list above + fresh `grep -rn "traveltour\|tevily\|traveltour-" src/ | grep -v plugins/theme_` — every hit mapped to a ThemeDef field or documented exception.
2. Extend `ThemeDef` + `theme_for_stylesheet`; thread the resolved theme through `SiteCtx`/render ctx (resolve once per site at boot from `settings.stylesheet`, cache on `SiteCtx` — re-resolve on settings change like other caches).
3. Migrate each call site (mechanical, one grep hit at a time).
4. `themes/specs/` loader + codegen in build.rs (copy `plugins/specs` pattern).
5. `templates/theme/default/` minimal templates — semantic HTML5, no brand classes.
6. Tevily TOML + verify against existing tevily fixture site; recapture visual_smoke fixture only if markup intentionally changed.
7. Update `docs/architecture.md` (theme contract section) + README ("new theme = TOML + templates dir").

## Success Criteria

- `grep -rn "traveltour" src/ | grep -v "plugins/theme_traveltour"` → empty.
- Tevily site renders through TOML theme; visual_smoke tevily gate passes (recaptured fixture if class names changed by design).
- Dropping a new `themes/specs/foo.toml` + `templates/theme/foo/` onboards theme foo with zero `src/` edits.
- `cargo test` + `check-layers.sh` green.

## Risk Assessment

- `cat_path`/`tax_lang` as `fn` pointers in TOML-generated defs — generate closures matching the signature; anything beyond "prefix + slug" needs Rust adapter (documented escape hatch).
- Caching theme on `SiteCtx`: `settings.stylesheet` can change via admin — invalidate on settings write or keep the existing 30s settings-snapshot TTL semantics.
- Default theme fallback could mask a misconfigured stylesheet — log at resolve time: `theme resolved: traveltour|default (stylesheet=…)`.
- visual_smoke byte-parity fixtures: class-name changes WILL break fixtures — recapture is expected and documented; keep structural gates.
