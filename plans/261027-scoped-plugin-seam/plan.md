---
title: "Core/plugin seam — adapter contract, optional PHP bridge"
description: "Make rpress core consistent for every site type (post/page/media/comments/menus/taxonomy). Plugins integrate through ONE adapter contract with pluggable backends — Rust-native, data-layer, static-replay, or an opt-in PHP bridge (php-wasm / Playground / php-cgi + sqlite-database-integration). Runs AFTER 261027-0310-audit-remediation."
status: pending
priority: P2
effort: "2-3w"
tags: [architecture, plugins, php-bridge, adapter]
created: 2026-10-27
---

# Core/plugin seam — adapter contract

## Premise (user-confirmed 2026-10-27)

Core must be identical for every kind of site. WP plugins/themes are
**inherited**, not rewritten per-site — but "inherited" has three real
forms, and the right one depends on the plugin:

- **Core** (always present): posts (kind `post`/`page`), pages, media,
  comments, menus, widgets, taxonomy, users/sessions, import → generate
  → serve → deploy. Public path stays 100% Rust + static files.
- **Plugin contract**: every plugin registers through ONE
  `PluginAdapter` trait. The adapter's *backend* is swappable per
  plugin — that is the whole point.

## The adapter contract

```rust
trait PluginAdapter {
    fn slug(&self) -> &str;                          // "goodlayers"
    fn kinds(&self) -> &[KindDef];                   // optional: custom post kinds, url_base
    fn admin_screens(&self) -> Vec<ScreenDef>;       // /admin/plugins/<slug>/…
    fn import_hooks(&self) -> Option<ImportHooks>;   // wp_posts/options → plugin tables
    fn render(&self, conn, post) -> Option<Html>;    // optional static render
    fn backend(&self) -> Backend;
}

enum Backend {
    Rust,          // native port — tourmaster, submissions (already exist)
    Data,          // read/write plugin DATA only (builder tree, options,
                   // meta) — output stays the already-rendered HTML.
                   // gdlr_pb should shrink to this, not a 2400-line
                   // engine port.
    StaticReplay,  // serve captured plugin output; read-only
    PhpBridge,     // opt-in — forwards to a real WP running the plugin
}
```

## Backend: PhpBridge (the only PHP that ever exists)

Used ONLY for JS-editor-heavy plugins (Elementor, WPBakery, Divi…) whose
editor UI cannot be economically reproduced.

### Runtime order — LEAST trusted-code first

1. **`Data` backend whenever possible** — no PHP at all (gdlr_pb target).
2. **php-wasm + WP-API shim (target runtime)** — the plugin's PHP runs
   inside a WASM sandbox: no raw syscalls, no network, no filesystem
   beyond what the shim exposes. The shim is a small WP-API surface
   (`get_post_meta`, `update_post_meta`, `wp_nonce_*`, `apply_filters`…)
   backed directly by `rpress.db`. Strongest isolation; highest effort.
3. **`@wp-playground/cli`** — real WP in php-wasm + SQLite; the PHP is
   WASM-sandboxed but the Node host process is extra surface; spawn
   on demand, localhost only.
4. **`php-cgi` native (last resort)** — real PHP with full syscalls;
   MUST run under a dedicated OS user (or container), filesystem
   read-only except the plugin mount, `open_basedir` restricted,
   `disable_functions` for exec/network, per-request timeout + kill.

### Hard rules (non-negotiable)

- **Opt-in per plugin, per site** — a site with no PhpBridge plugins
  deploys exactly today's single binary; no PHP anywhere.
- **Admin-scoped, authed, localhost-only** — lives under
  `/admin/plugins/*` behind the rpress admin session; the bridge
  binds 127.0.0.1 and is never exposed to the public network.
- **On-demand + TTL** — spawned on first plugin-screen request,
  killed after an idle timeout (default ~10 min); zero resident
  cost when unused.
- **Write whitelist (source of truth)** — `rpress.db` is the ONLY
  primary store. Bridge code may NEVER write to it directly;
  `adapter.write()` accepts only the plugin's own namespace
  (its meta keys, its tree/options tables) and rejects anything
  touching `posts`, `users`, `sessions`, `settings`, or other
  plugins' data. A compromised plugin can dirty its own data at
  worst — never the core.
- **Blast radius documented** — a vulnerability in the PHP layer
  cannot reach public visitors (static serve) but CAN reach
  rpress.db writes and the admin session; the whitelist + sandbox
  above are the mitigations, not "it's only admin".
- **Two-source-of-truth ban** — never let a bridge WP become
  primary (the Strattic failure: drift + divergence).

## Plugin inventory → backend assignment

| Plugin | Backend | Note |
|---|---|---|
| tourmaster | Rust | already `admin/plugins/tourmaster/`; move `kind='tour'`/`tours` table/`tour_base`/`tour__*.sql`/`widgets__recent_tours`/`sidebar-tour-*` seeds behind adapter registration |
| contact forms / intake / submissions | Rust | already ported (intake.rs, forms_markup.rs) |
| rankmath | Rust or Data | small |
| goodlayers (gdlr_pb) | **Data** | shrink 2404-line engine port → tree editor over body_json + replay rendered HTML; keep `html_decode` helper. User confirmed: porting the engine was a mistake |
| Elementor / WPBakery / Divi (future) | PhpBridge | real editor via bridge; write lands through adapter |
| themes (CAT_TREE/etiquette/ES_ROOTS) | Data | theme = templates + seed + taxonomy map as data, not hardcoded Rust |

## What core must change to support this

- `kind='tour'` → registered via adapter (kind registry).
- `tour_base`/`TOUR_BASE` → adapter-provided config key, not core SiteEnv.
- Import: post-kind inserts dispatch through adapter `import_hooks`
  instead of hardcoded `tours` table writes.
- Admin router: `/admin/plugins/<slug>` dispatches to the adapter.
- Serve: public path NEVER loads adapters — it only reads files.

## Performance contract

- Public request: unchanged — `fs::read`. No adapter code runs.
- Admin plugin screen via PhpBridge: one WP request (~100–500ms);
  idle cost 0 when spawned on demand.
- Binary size: unchanged unless php-wasm is compiled in (opt-in).

## Competitive scan (2026-10-27)

| Who | Approach | Lesson for rpress |
|---|---|---|
| Strattic (Elementor) | WP staging + static publish | Died 2025 — publish-step divergence + forever maintaining WP. rpress avoids this: rpress.db is source of truth, WP is optional tooling |
| Simply Static | WP plugin → static export | WP stays alive as admin = same debt rpress escapes; no dynamic features (comments/forms/search) — rpress already ported those |
| wordpress-playground / php-wasm | WP in WASM + SQLite | **Provides the missing piece**: cheap real-WP sidecar without a PHP stack — use as PhpBridge runtime |
| sqlite-database-integration | WP on SQLite (official) | Bridge WP needs no MySQL |
| EmDash (Cloudflare) | TS/Astro "WP successor" + plugin registry | Competes on ecosystem; rpress differentiator = zero-runtime public + migration parity + wp-admin UX |

## Risks

- PHP sidecar = a PHP runtime must be maintained per-site that uses it
  (patch cadence is slow since non-public, but nonzero).
- Two-source-of-truth drift if bridge WP writes its own posts —
  adapter `write()` MUST funnel through rpress.db.
- Data-backend plugins need a per-plugin map (config, not engine) —
  still per-plugin work, but bounded.
