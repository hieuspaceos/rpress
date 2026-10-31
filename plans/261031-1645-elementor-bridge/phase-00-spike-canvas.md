# Phase 00 — Spike

Evidence from Elementor `editor.php`: shell is `post.php?post=&action=elementor`.

1. Temporarily allow `?` on bridge HTTP.
2. Stub adapter slug `elementor` so the mount exists.
3. Blueskies `PLUGINS_PHP=elementor`, GET `contact-2`’s wp_id (page 20 / similar).
4. Record: HTML 200? 403? REST 404? iframe `/?p=` 200? php.log deadlock?

Go = shell HTML contains `#elementor-preview` (`editor-body-view.php`). No-go = deadlock or 403 → runtime change, not adapter work.
