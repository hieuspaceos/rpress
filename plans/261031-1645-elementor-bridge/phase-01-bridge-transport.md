# Phase 01 — Shared transport

- Query forward; path still no `..`/CRLF/`//`.
- Rewrite html+json+js WP_HOME → `/admin/plugins/{slug}`.
- WP_HOME/SITEURL/PLUGIN_URL = rpress mount.
- `router.php` for php -S (non-file → `index.php`).
- `PHP_CLI_SERVER_WORKERS=4`.
- Tests: query accept/reject; GDLR ajax still 200.
