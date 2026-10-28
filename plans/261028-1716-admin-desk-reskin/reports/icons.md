# Icon inventory — Atrium reskin (Phase 1)

Source: `grep -rn 'dashicons-' src/admin` (emitted markup) + `templates/admin/wp/*.js`
(custom first-party scripts — none emit dashicons). Vendored WP CSS/JS was
excluded; dashicons.woff stays loaded as fallback for unmapped classes.

## Menu icons (src/admin/mod.rs chrome_start + plugins)

| Class | Used for | Tint (--icon-tint) | Custom glyph |
|-------|----------|--------------------|--------------|
| dashicons-dashboard | Dashboard | amber | Y |
| dashicons-admin-post | Posts | coral (accent) | Y |
| dashicons-admin-media | Media | violet | Y |
| dashicons-admin-page | Pages | teal | Y |
| dashicons-admin-comments | Comments | lime | Y |
| dashicons-star-filled | Site Reviews | amber | Y |
| dashicons-email | Contact | coral | Y |
| dashicons-admin-appearance | Appearance (Menus/Widgets) | violet | Y |
| dashicons-admin-users | Users | teal | Y |
| dashicons-admin-settings | Settings | muted/amber | Y |
| dashicons-palmtree | Tourmaster (tours) | ok/teal | Y |
| dashicons-tickets-alt | plugin spec default | ok | Y |
| dashicons-admin-generic | plugin fallback | muted | Y |
| dashicons-admin-post / -page | plugin kinds reuse | — | same glyphs |

## In-screen icons

| Class | Site | Custom glyph |
|-------|------|--------------|
| dashicons-visibility | auth.rs:49 login show-password | Y (eye) |
| dashicons-translation | auth.rs:62 login language | Y (globe) |
| dashicons-arrow-down | menus.rs:275,296 accordion triggers | Y (chevron) |
| dashicons-media-default | admin.rs:1399,1530 media preview placeholder | Y |
| dashicons-format-image / -audio / -video | admin.rs:946-948 mime icons | Y |
| dashicons-pdf | admin.rs:950 | Y |
| dashicons-editor-textcolor | admin.rs:951 font files | Y |
| dashicons-before | mod.rs:1177 menu icon container prefix | N (container only) |

## JS-toggled / dynamic

- `classList` scan of first-party JS (menu-editor.js, svg-painter.js,
  common.min.js hover only): no dashicons classes toggled.
- `dashicons-before` is a structural prefix on `.wp-menu-image`, not a glyph —
  the glyph comes from the paired class (`{icon}` arg).
- `wp-menu-image` contains `<br>` only; glyph is `::before` font content on
  the div. Override = `content:none` + CSS mask data-URI.
- `#collapse-button .collapse-button-icon` is WP's pseudo-element arrow —
  repurpose as dock collapse/expand affordance (mapped, Y).

## Notes

- ~14 glyph set needed for menus + ~9 in-screen ≈ 23 total, inside the
  <8KB data-URI budget.
- Fallback policy: `content:none` applied ONLY to the mapped classes above.
