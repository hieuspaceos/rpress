# Audit — Atrium reskin (Phase 1)

## #adminmenu DOM (src/admin/mod.rs:1176-1190, chrome_start)

```
#wpwrap > #adminmenumain > #adminmenuback (empty sibling)
                    > #adminmenuwrap > ul#adminmenu
```

Each item (`menu_li`):
```html
<li id="menu-{id}" class="wp-has-submenu wp-has-current-submenu wp-menu-open menu-top current | menu-top">
  <a href="{href}" class="menu-top {aclass}">
    <div class="wp-menu-image dashicons-before {icon}"><br></div>
    <div class="wp-menu-name">{label}{badge-bubble}</div>
  </a>
  <ul class="wp-submenu wp-submenu-wrap">
    <li class="wp-submenu-head" aria-hidden="true">{label}</li>
    <li class="wp-first-item"><a href="…">…</a></li> …
  </ul>
</li>
<li id="collapse-menu" class="hide-if-no-js"><button id="collapse-button">…</button></li>
```

Top-level order: dashboard | posts(sub: All Posts, Add Post, Categories,
Tags) | media(sub: Library, Add New) | pages(sub: All Pages, Add Page) |
comments(badge, no sub) | reviews | plugin kinds (tours etc.) |
contact(sub: Forms, Submissions) | menus=Appearance(sub: Menus[, Widgets]) |
users(admin, sub) | settings(admin, 7-item sub) | collapse-menu.

Badges: `.awaiting-mod.count-N > .pending-count` inside `.wp-menu-name`.

## Dock flyout plan

- `#adminmenu` → bottom-center horizontal pill; each `li.menu-top` → ~52px
  tile (icon + hover label); `.wp-menu-name` hidden, label surface in tile.
- `.wp-submenu.wp-submenu-wrap` → `position:fixed` glass flyout, bottom
  anchored above the dock; `.wp-submenu-head` hidden.
- `.awaiting-mod` → absolute badge on the tile.
- `#collapse-menu` → dock "+" / hide tile; keep button functional.

## cms-overrides.css catalogue (templates/admin/wp/cms-overrides.css)

| Lines | Rule | Constraint for reskin |
|-------|------|------------------------|
| 7-16 | `#adminmenuwrap { position:fixed !important; top:32px; bottom:0; left:0; overflow-y:auto; overflow-x:hidden }` | Dock MUST use `!important` to repaint; overflow clip requires `.wp-submenu` `position:fixed` |
| 21-34 | `.tablenav.top` flex layout (bulk + search + pager one row) | Keep flex; reskin fields only |
| 38-49 | `#att-modal > div` sized, `#am-preview img` max-height | Scoped `!important` waiver zone |
| 54-67 | `#wp-content-editor-tools.wp-pin` top:32px; `.mce-toolbar-grp.wp-pin` top:68px | Adminbar must stay exactly 32px |
| 73-85 | `#wp-content-editor-tools` flex (media-buttons + tabs) | Repaint only |
| 90-92 | `.quicktags-toolbar` hidden | — |
| 100-131 | `.mce-content-body` image/caption layout (also in TinyMCE via content_css) | Atrium content_css must not disturb these; appended last |
| 136-142 | `#postbox-container-1.wp-pin` fixed, top:32px, z-index:50 | Offsets verified |

## Breakpoints (WP core, unmodified)

- auto-fold <960px (body.auto-fold already set; fold toggles on resize)
- mobile collapse <782px (`wp-responsive-open`, `mobile` classes)

## CSS load order (html_admin, mod.rs:942-970)

dashicons → admin-bar → common → forms → admin-menu → dashboard →
list-tables → edit → editor → media → nav-menus → l10n → admin-schemes →
wp-tooltip → buttons → colors → **cms-overrides → atrium.css (new, last)**

Login (auth.rs:8-21): dashicons → buttons → forms → l10n → admin-schemes →
wp-tooltip → login.min.css → **atrium.css (new)**. No cms-overrides,
no colors.min.css, no common.min.js — login rules self-contained.

JS: jquery, hoverintent, common.min.js (writes inline `margin-top` on
`.wp-submenu` — overridden via `margin-top:0 !important` in dock scope),
admin-bar, svg-painter. New: `atrium.js` deferred at `chrome_end`.

## Screens to cover (route map)

login, dashboard (/admin), posts list, post edit (TinyMCE), pages, media
library + upload modal (#mf-frame) + attachment modal (#att-modal), menus,
comments, users, settings×7, widgets, revisions, taxonomies (/admin/tax/*),
contact (forms + inbox), Rank Math cells, plugin screens (/admin/plugins/*,
tours), site reviews, post edit preview.

## Drift vs plan (found during audit)

- `content_css` exists at **2 sites, not 3**: posts.rs:868 and
  widgets.rs:648 (plan cites posts.rs:868,1043 + widgets.rs:631 — line drift;
  posts.rs:1043 no longer exists as a third site).
- `rpress site serve --smoke` exists (verified `--help`).
