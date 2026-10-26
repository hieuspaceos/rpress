WP 7.1.3 (test clone, 2026-10-22)

Vendor: `templates/admin/wp/` from ct4 `test.aucoeurvietnam.com`. Hashes in `SHA256SUMS`.

Login stylesheets (order, no `colors.min.css`):
1. dashicons.min.css
2. buttons.min.css
3. forms.min.css
4. l10n.min.css
5. admin-schemes.min.css (`wp-base-styles`)
6. wp-tooltip.min.css
7. login.min.css

Login JS: jquery.min.js, user-profile.min.js (show password). Do not load common.min.js on login.

Logged-in extra (phase 2): admin-bar, common, admin-menu, dashboard, list-tables, edit, media, nav-menus, colors.min.css (Modern), hoverintent-js, common.min.js, admin-bar.min.js, svg-painter.js.

Modern tokens (`admin-color-modern`, `admin-schemes.min.css` + `colors/modern/colors.min.css`):
- `$base-color` / menu: `#1e1e1e`
- `$highlight-color` / `--wp-admin-theme-color`: `#3858e9`
- submenu focus: `#7b90ff`

Not Fresh: `#1d2327`, `#2271b1`, `#72aee6`.

Form `name="log"` is the CMS `users.email` value. Do not rename the column.
