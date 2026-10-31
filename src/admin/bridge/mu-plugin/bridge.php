<?php
/**
 * rpress bridge mu-plugin — loaded inside the bridge WP on every request.
 *
 * Jobs:
 *  1. Validate the one-time bearer token issued by rpress (X-Rpress-Bridge
 *     header vs the RPRESS_BRIDGE_TOKEN constant the spawn passes via
 *     `--define` — process env does NOT propagate into PHP-WASM).
 *     A mismatch gets 403 before WordPress does anything — the bridge URL
 *     is localhost-only already; this is the second fence.
 *  2. Run admin requests as the install's admin user (uid 1) so `wp_ajax_*`
 *     hooks fire — admin-ajax only fires `wp_ajax_nopriv_*` for guests and
 *     plugin endpoints would die('0'). The token above is the auth
 *     boundary; inside the bridge, uid 1 is the only identity.
 *  3. One-time seed (first admin request): activate the plugin declared
 *     by the RPRESS_BRIDGE_PLUGIN constant and import the projected post
 *     from wp-content/rpress-seed/post.json so `get_post`/`get_post_meta`
 *     answer for the post being edited. wp.db is disposable; re-seeding
 *     on respawn is cheap and keeps rpress.db the only source of truth.
 *
 * Keep this file dependency-free: it runs before plugins load.
 */

/* Fence 1: bearer token — a missing constant means 403, never uid-1 admin. */
$tok = defined('RPRESS_BRIDGE_TOKEN') ? RPRESS_BRIDGE_TOKEN : '';
$got = isset($_SERVER['HTTP_X_RPRESS_BRIDGE']) ? $_SERVER['HTTP_X_RPRESS_BRIDGE'] : '';
if ($tok === '' || !hash_equals($tok, $got)) {
    http_response_code(403);
    header('Content-Type: text/plain');
    echo 'bridge token mismatch';
    exit;
}

/* Identity probe: rpress verifies the process answering on the bridge port
 * is OUR child, not a foreign listener that took the port after a restart.
 * Sits after the token fence — the bearer is already proven here. */
if (isset($_SERVER['REQUEST_URI']) && strpos($_SERVER['REQUEST_URI'], '/__rpress_ident') === 0) {
    http_response_code(204);
    header('X-Rpress-Ident: bridge');
    exit;
}

/* Identity: admin requests run as uid 1 (see header note 2). Also mint a
 * real nonce for gdlr ajax calls — our admin page can't hold a WP nonce,
 * and `check_ajax_referer('gdlr_core_page_builder')` would die('-1').
 * Runs after the token fence, so nonce-fakeability is bounded by the token. */
add_action('admin_init', function () {
    wp_set_current_user(1);
    if (!empty($_POST['action']) && strpos($_POST['action'], 'gdlr_core_') === 0) {
        $n = wp_create_nonce('gdlr_core_page_builder');
        $_POST['security'] = $n;
        $_REQUEST['security'] = $n;
    }
    /* html-option actions (`gdlr_get_gallery_options`, image fields inside
       element lightboxes) check the `gdlr_core_html_option` nonce — same
       mint path, different action name. */
    if (!empty($_POST['action']) && strpos($_POST['action'], 'gdlr_get_') === 0) {
        $n = wp_create_nonce('gdlr_core_html_option');
        $_POST['security'] = $n;
        $_REQUEST['security'] = $n;
    }
}, 0);

/* Self-seed once per wp.db lifetime: plugin activation + post import.
 * Runs inside admin context (admin-ajax.php / wp-admin/*) where
 * activate_plugin is available. */
add_action('admin_init', function () {
    if (get_option('rpress_seed_done')) {
        return;
    }
    update_option('rpress_seed_done', 1, false);

    $plugin = defined('RPRESS_BRIDGE_PLUGIN') ? RPRESS_BRIDGE_PLUGIN : '';
    require_once ABSPATH . 'wp-admin/includes/plugin.php';
    if ($plugin !== '') {
        $plugin_file = WP_PLUGIN_DIR . '/' . $plugin;
        if (file_exists($plugin_file)) {
            activate_plugin($plugin);
        }
    }
    /* Site parity: every plugin the origin had active — element/template
       classes (tourmaster tour/room elements, room pages) only exist while
       their plugin is. `RPRESS_BRIDGE_PLUGINS` is a JSON list of
       `dir/file.php` plugin paths, skipped silently when unmounted. */
    if (defined('RPRESS_BRIDGE_PLUGINS')) {
        foreach ((array) json_decode(RPRESS_BRIDGE_PLUGINS, true) as $p) {
            if (is_string($p) && $p !== $plugin && file_exists(WP_PLUGIN_DIR . '/' . $p)) {
                activate_plugin($p);
            }
        }
    }

}, 1);

/* The site's real theme, not the playground default — theme filters
   (e.g. `gdlr_core_skin_option_slug`, tourmaster element registration)
   only exist while it is active. Runs every request (cheap) so a bridge
   seeded before this rule still converges; theme functions.php loads
   from the NEXT request onward. */
add_action('admin_init', function () {
    $theme = defined('RPRESS_BRIDGE_THEME') ? RPRESS_BRIDGE_THEME : '';
    if ($theme !== '' && get_option('stylesheet') !== $theme) {
        switch_theme($theme);
    }

}, 1);

add_action('admin_init', function () {
    if (get_option('rpress_post_seeded')) {
        return;
    }
    $seed = WP_CONTENT_DIR . '/rpress-seed/post.json';
    if (file_exists($seed)) {
        $j = json_decode(file_get_contents($seed), true);
        /* `import_id` isn't a WP dedupe key — guard on the row existing so
           re-seeds (fresh wp.db sharing post.json) can't double-insert. */
        $p = is_array($j) ? ($j['post'] ?? null) : null;
        if (is_array($p) && (empty($p['ID']) || !get_post((int) $p['ID']))) {
            $pid = wp_insert_post([
                'import_id'    => isset($p['ID']) ? (int) $p['ID'] : 0,
                'post_title'   => $p['post_title'] ?? '',
                'post_content' => $p['post_content'] ?? '',
                'post_excerpt' => $p['post_excerpt'] ?? '',
                'post_status'  => $p['post_status'] ?? 'draft',
                'post_name'    => $p['post_name'] ?? '',
                'post_type'    => $p['post_type'] ?? 'page',
            ], true);
            if (!is_wp_error($pid)) {
                update_option('rpress_post_seeded', $pid, false);
                if (!empty($j['meta']) && is_array($j['meta'])) {
                    foreach ($j['meta'] as $mk => $mv) {
                        update_post_meta($pid, $mk, $mv);
                    }
                }
            }
        }
    }
}, 1);

/* One-shot dump for rpress tooling: element/wrapper defs as JSON so the
 * Rust canvas can mirror the plugin's own templates/settings. Token-fenced
 * like everything else; admin action => wp_ajax_. */
add_action('wp_ajax_rpress_dump_defs', function () {
    if (!class_exists('gdlr_core_page_builder_element') || !class_exists('gdlr_core_page_builder_wrapper')) {
        wp_send_json_error('pb classes missing');
    }
    $rp = new ReflectionProperty('gdlr_core_page_builder_element', 'pb_elements');
    $els = $rp->getValue();
    $out = ['elements' => [], 'wrappers' => []];
    foreach ($els as $slug => $class) {
        $entry = [];
        foreach (['get_settings' => 'settings', 'get_options' => 'options', 'get_template' => 'template'] as $m => $key) {
            try { $entry[$key] = call_user_func($class . '::' . $m); }
            catch (Throwable $e) { $entry[$key . '_error'] = $e->getMessage(); }
        }
        $out['elements'][$slug] = $entry;
    }
    $rw = new ReflectionProperty('gdlr_core_page_builder_wrapper', 'pb_elements');
    foreach ($rw->getValue() as $slug => $class) {
        try {
            $out['wrappers'][$slug] = [
                'settings' => call_user_func($class . '::get_settings'),
                'options'  => call_user_func($class . '::get_options'),
                'template' => call_user_func($class . '::get_template'),
            ];
        } catch (Throwable $e) {
            $out['wrappers'][$slug] = ['error' => $e->getMessage()];
        }
    }
    // template pieces each section contributes to page_builder_var
    $pbv = [];
    foreach ((array)(new ReflectionProperty('gdlr_core_page_builder','pb_section'))->getValue() as $sec => $cls) {
        try { $pbv = call_user_func($cls . '::set_page_builder_var', $pbv); }
        catch (Throwable $e) { $pbv['errors'][$sec] = $e->getMessage(); }
    }
    $out['page_builder_var'] = $pbv;
    wp_send_json($out);
});
