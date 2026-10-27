---
title: "Engagement widgets — popups, floating buttons, native chat"
description: "Core subsystem for site-wide engagement UI: popups (delay/scroll/exit-intent, page targeting, frequency caps), floating action buttons (call/Zalo/WhatsApp/messenger/scroll-top), and native chat widget (own inbox or third-party embed). Config-driven in admin, zero code per site."
status: pending
priority: P2
effort: "1-2d"
tags: [widgets, popup, chat, engagement, core]
created: 2026-10-27
---

# Engagement widgets

## Premise (user-confirmed 2026-10-27)

Every news/content site needs engagement UI: announcement popups,
floating contact buttons, chat. This belongs in **core** (universal,
not a plugin). It must be pure config + static output — public path
stays fs::read, so all rendering is template + injected markup at
generate/render time or a small static JS snippet.

## Current state

- No popup/chat code today (verify — grep popup|chat|floating).
- Widget/sidebar system exists (`src/widgets.rs`, seed JSON) — but these
  are *sidebars*, not floating/global UI.
- `templates/` minijinja + `front.css`/`nav.js` in `www/` — natural home
  for the JS snippet (served static, no runtime).

## Components

| Component | What it is | Config |
|---|---|---|
| **Popup** | modal/banner injected into page chrome | trigger (delay s / scroll % / exit-intent), page targeting (all/path-prefix/post-kind), frequency (once/session/N days via localStorage), content HTML (minijinja or raw) |
| **Floating button** | fixed FAB bottom-right/left, expandable actions | icon + label + href per action; presets: tel:, zalo.me/, wa.me/, m.me/, scroll-top, back |
| **Chat native** | (a) simple chatbox → stores messages into submissions table (reuse intake inbox) — "offline chat"; (b) third-party embed passthrough (tawk.to/crisp/messenger customer chat script snippet) | mode select: native-inbox | embed-script |

All rendered as static markup + one `engagement.js` (~small, vanilla)
emitted by `sitefx`/generate — no server state needed except native
chat messages, which reuse the submissions intake path.

## Wiring

- `sitefx.rs` (or generate chrome path): emit `engagement.js` + inject
  markup block into base template when configured.
- Admin settings screen: per-component enable + fields; preview note.
- Config storage: `settings` table keys (`engagement_popup`, …) or a
  dedicated `engagement` table if targeting needs rows.

## Non-goals

- Real-time websocket live chat (that's a service, not a static-site
  feature — use embed mode for tawk.to/crisp).
- A/B testing, analytics dashboards.

## Success criteria

- [ ] Popup renders per trigger rules without server code.
- [ ] Floating button with ≥3 preset actions renders + clicks through.
- [ ] Native chatbox submits → lands in submissions inbox.
- [ ] Embed mode pastes a third-party script verbatim safely.
- [ ] Disabled = zero markup emitted (no dead JS).
- [ ] `cargo check` + `cargo test` clean.

## To be phased by ak:plan

Outcome contract only — needs scout (template chrome injection point,
submissions reuse surface, JS serving path) → phases → red-team.
