---
phase: 4
title: "Aucoeur GDLR builder regression check"
status: done
priority: P2
effort: "1h"
dependencies: [1]
---

# Phase 4: Aucoeur GDLR builder regression check

## Goal

Uniform `wp_slash` on seed meta must not corrupt the GDLR path:
PHP-serialized `gdlr-core-page-builder` must still unserialize, and the
aucoeur tour edit screen must open with the real builder.

## Facts

- Seed `gdlr-core-page-builder` is PHP-serialized (`a:19:{...}`), no `\`
  or `\u` sequences → wp_slash adds slashes that unslash removes:
  identity transform. Still, serialized STRINGS inside contain quotes —
  unslash must restore them exactly; verify byte-identical round trip.
- Earlier flake observed: `edit.php?post_type=tour` 403'd once with a
  still-seeding wp.db — retry after full seed before calling it a bug.

## Steps

1. Serve aucoeur-test with `PLUGINS_PHP=goodlayers`; delete
   `www/bridge/goodlayers` to force re-seed with fixed mu-plugin.
2. `sqlite3 wp.db "SELECT meta_value FROM wp_postmeta WHERE
   post_id=6416 AND meta_key='gdlr-core-page-builder'"` →
   `php -r 'unserialize(...)'` parses; compare to seed value byte-exact.
3. Browser: login (`work/aucoeur-test/credentials.txt`), open
   `/admin/plugins/goodlayers/wp-admin/post.php?post=6416&action=edit`.
4. Assert GDLR page builder UI renders sections (screenshot); make a
   trivial text edit in a text element, save, confirm it lands in
   wp.db and is pulled into `rpress.db` (post_meta + body_json via the
   BodyJson spec).
5. Revert the edit; leave clean state.

## Verification

- Serialized meta byte-exact between post.json and wp.db.
- Builder shows multi-section layout, edit persists post-pull.
- `cargo test --lib` green at end.
