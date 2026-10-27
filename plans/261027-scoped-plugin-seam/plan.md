---
title: "Core/plugin seam — adapter contract, optional PHP bridge"
description: "Make rpress core consistent for every site type (post/page/media/comments/menus/taxonomy). Plugins integrate through ONE adapter contract with pluggable backends — Rust-native, data-layer, static-replay, or an opt-in PHP bridge (php-wasm / Playground / php-cgi + sqlite-database-integration). Runs AFTER 261027-0310-audit-remediation."
status: pending
priority: P2
effort: "2-3w"
branch: main
tags: [architecture, plugins, php-bridge, adapter]
created: 2026-10-27
---

# Core/plugin seam — adapter contract

## Premise (user-confirmed 2026-10-27)

Core must be identical for every kind of site. WP plugins/themes are
**inherited**, not rewritten per-site — but "inherited" has three real
forms, and the right one depends on the plugin:

- **Core** (always present): posts (kind `post`/`page`), pages, media,
  comments, menus, widgets, taxonomy, users/sessions, **SEO**
  (Rank Math-parity meta/og/schema/sitemap — universal to every site),
  import → generate → serve → deploy. Public path stays 100% Rust +
  static files.
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
    Data,          // read/write plugin DATA only (options, meta, trees
                   // without an editor engine) — output stays the
                   // already-rendered HTML.
    StaticReplay,  // serve captured plugin output; read-only
    PhpBridge,     // opt-in — forwards to a real WP running the plugin
}
```

The existing per-kind editor seam `EditorPlugin`
(`src/admin/plugins/mod.rs:18-31`, registry `PLUGINS` at
`src/admin/plugins/mod.rs:33`) is subsumed: `KindDef.editor` carries the
same load/metabox/assets/save hooks, and `PluginAdapter` adds screens,
import hooks, render, and the backend label.

## Backend: PhpBridge (the only PHP that ever exists)

Used ONLY for JS-editor-heavy plugins (Elementor, WPBakery, Divi…) whose
editor UI cannot be economically reproduced.

### Runtime order — LEAST trusted-code first

1. **`Data` backend whenever possible** — no PHP at all.
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
  worst — never the core. **Enforcement is adapter-side, never
  PHP-side** (see phase 06 + red-team R3).
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
| contact forms / intake / submissions | Rust | already ported (`src/intake.rs`, `src/forms_markup.rs`, `src/db/submissions.rs`) — stays core; no adapter needed |
| rankmath / SEO | **Core** | universal — every site needs meta/og/schema/canonical; `src/admin/rankmath.rs` (666 lines) stays core, no adapter needed |
| goodlayers (gdlr_pb) | **PhpBridge** | REPLACE — editors use the Page Builder metabox daily; the 2412-line Rust port (`src/admin/gdlr_pb.rs`) is deleted once the bridge path lands. Keep `html_decode` (used by public render at `src/render.rs:2019`) as a standalone helper |
| Elementor / WPBakery / Divi (future) | PhpBridge | real editor via bridge; write lands through adapter |
| themes (CAT_TREE/etiquette/ES_ROOTS) | Data | theme = templates + seed + taxonomy map as data, not hardcoded Rust (`src/render_post.rs:451`, `:581`; `etiquette` literals `src/render.rs:294`, `src/render_archive.rs:315`) |

## Phases

| # | Phase | Depends on | Effort |
|---|-------|-----------|--------|
| 01 | [Contract: PluginAdapter + kind registry + settings namespaces](./phase-01-adapter-contract.md) | audit phases 1–7 | 2d |
| 02 | [Tourmaster adapter registration](./phase-02-tourmaster-adapter.md) | 01 | 1.5d |
| 03 | [Tour extraction: render/generate/lang/widgets decoupling](./phase-03-tour-extraction.md) | 02 | 3d |
| 04 | [Theme-as-data: CAT_TREE / etiquette / ES_ROOTS / seeds](./phase-04-theme-data.md) | 01 | 2d |
| 05 | [Back-compat & migration for existing sites](./phase-05-backcompat-migration.md) | 02–04 | 1d |
| 06 | [PhpBridge runtime MVP (php-cgi or Playground) + write whitelist](./phase-06-phpbridge-runtime.md) | 01 | 3d |
| 07 | [gdlr_pb → PhpBridge cutover](./phase-07-gdlr-bridge.md) | 06 | 3d |

Total ≈ 15.5 dev-days. Phases 03 and 04 are independent of each other
once 02 lands; 05 can overlap 04; 06–07 are strictly sequential.

## Dependency graph

- **01 → everything**: the contract + kind registry is the substrate.
- **02 → 03**: tourmaster must be an adapter before its render/import
  logic can move behind `kinds()`/`import_hooks()`.
- **01 → 04**: theme data keys off adapter `Data` backend config.
- **05 after 02–04**: verifies every extraction against real DBs.
- **06 → 07**: gdlr cutover needs a running bridge.
- Public serve path is never touched by any phase (hard rule); each
  phase's verification re-asserts `fs::read`-only on `/{*public}`
  (`src/admin/mod.rs:241` route; `read_out` at `src/admin/mod.rs:657`).

## Red-team findings & dispositions

| # | Finding | Severity | Disposition |
|---|---------|----------|-------------|
| R1 | Kind registry vs `posts.kind` string column: unknown kinds silently fall back via `spec_for_kind` (`src/admin/posts.rs:39-41`, `.unwrap_or(&KINDS[0])` masks typos/missing plugins) | High | Registry returns `Option`; core kinds `post`/`page` built-in; unknown kind → explicit 404/empty-list, never silent post-ification. Covered in phase 01, step 4. |
| R2 | `tour_base` threading was just completed (audit phase 4); ripping it out mid-plan re-breaks 25+ call sites | High | Two-step migration: keep `tour_base` param threading; phase 03 only changes *who supplies the value* (kind registry `url_base` from plugin config) at the top-level entry points. Call sites below the entry points stay untouched. |
| R3 | Write whitelist enforced inside the bridge WP (mu-plugin) is bypassable — compromised PHP code could open rpress.db directly | High | Whitelist lives in the *adapter* (Rust side): bridge WP gets its own SQLite file; rpress reads back only declared keys via `adapter.write()` pull, or the bridge POSTs to an rpress endpoint that validates table+key prefix before executing. PHP never holds a path/handle to rpress.db. Phase 06 §write-path. |
| R4 | Import dispatch ordering: `kind_of` (`src/import.rs:294-302`) maps wp `post_type`→kind inline; plugins must claim types before the hardcoded match runs, or a plugin type is silently dropped/skipped | Med | Replace `kind_of` body with registry lookup first, core fallback second; unknown types keep today's `continue`+log. Phase 03 step 2. |
| R5 | `/admin/plugins/{slug}` route conflicts with catch-alls `/admin/{section}` (`src/admin/mod.rs:191`) and `/{*public}` (`src/admin/mod.rs:241`) | Med | Route order + axum specificity: `/admin/plugins/{slug}/{*path}` registered before `/admin/{section}`; `plugins` is a reserved section name rejected by `spec_for_section`. Phase 02 step 4. |
| R6 | gdlr ajax nonce: vendored page-builder.js sends WP `security` nonces; bridge WP will issue WP-format nonces rpress can't validate — and vice versa during transition | Med | Bridge endpoint terminates auth itself: rpress session cookie is NOT forwarded; bridge issues its own nonce to the JS via the boot shim (`page_builder_var` rewrite at `src/admin/gdlr_pb.rs:1794-1801`). Phase 07 step 3. |
| R7 | `promote_tours` runs on every admin boot (`src/admin/mod.rs:87`) scanning all tours — plugin extraction must not regress boot | Low | Move behind adapter `import_hooks`/`activate`; same single pass, but only when a registered plugin claims `tour` kind. Phase 02 step 5. |
| R8 | `posts__insert_into_tours*` SQL files + `tours` table in core `DDL` (`src/schema.rs:26-36`) — plugin-owned table in core schema means `site_init` sites without tourmaster still get the table | Low | Plugin DDL moves to `schema.rs` of the tourmaster adapter, applied by adapter `migrate()` at registration; core keeps `CREATE TABLE IF NOT EXISTS` for back-compat on existing DBs (phase 05). |
| R9 | `Data`-backend themes still need per-site taxonomy maps; CAT_TREE is a frozen source-site snapshot — making it data risks editors changing URL layout unknowingly | Med | Theme data is *seed-time* data (imported once, stored in settings/terms rows), not a live-editable config. URL layout stays deterministic; editor changes go through normal term editing. Phase 04. |
| R10 | Playground/php-cgi availability on deploy hosts can't be assumed; bridge must degrade, not break admin | Med | Adapter `backend()==PhpBridge` but runtime unavailable → admin screen renders "PHP runtime not installed; install X to edit with GoodLayers" instead of erroring. `body_json` save path (`src/db/misc.rs:868` `body_is_gdlr`, `src/db/posts.rs:1039`) still works via raw JSON. Phase 06 step 6. |

## Risks

- PHP sidecar = a PHP runtime must be maintained per-site that uses it
  (patch cadence is slow since non-public, but nonzero).
- Two-source-of-truth drift if bridge WP writes its own posts —
  adapter `write()` MUST funnel through rpress.db.
- Data-backend plugins need a per-plugin map (config, not engine) —
  still per-plugin work, but bounded.

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

## What core must change to support this (summary)

- `kind='tour'` → registered via adapter (kind registry) — phases 01–03.
- `tour_base`/`TOUR_BASE` → adapter-provided config key, not core SiteEnv — phase 03.
- Import: post-kind inserts dispatch through adapter `import_hooks`
  instead of hardcoded `tours` table writes — phases 02–03.
- Admin router: `/admin/plugins/<slug>` dispatches to the adapter — phase 02.
- Serve: public path NEVER loads adapters — it only reads files — all phases.

## Validation sweep (2026-10-27)

- Contradictions vs current code: 0. Every `file:line` citation above
  re-verified against working tree on 2026-10-27.
- Audit-plan overlap: phases 6–7 of `261027-0310-audit-remediation` are
  in flight. This plan's phases 01–03 touch `src/admin/posts.rs`,
  `src/db/posts.rs`, `src/admin/mod.rs` — sequencing rule: seam work
  starts only after audit phases 6+7 merge; phase files note the exact
  symbols to re-verify at start.
