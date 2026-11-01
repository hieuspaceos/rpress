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
/* auth_redirect() (wp-admin/admin.php) requires a cookie; we have the
   bearer instead. Override pluggable fn from mu-plugin (loads first).
   admin-ajax (GDLR) never calls auth_redirect. */
function auth_redirect() {
    if (function_exists('wp_set_current_user')) {
        wp_set_current_user(1);
    }
}

/* Force-load the bridge plugin on THIS request. `activate_plugin` during
   WP_INSTALLING leaves active_plugins=a:0:{}; `option_active_plugins` +
   file_exists() can miss because WP_PLUGIN_DIR is not always set yet.
   `pre_option_*` short-circuits the DB. GDLR still listed via
   RPRESS_BRIDGE_PLUGIN. */
add_filter('pre_option_active_plugins', static function () {
    $v = [];
    if (defined('RPRESS_BRIDGE_PLUGIN') && RPRESS_BRIDGE_PLUGIN !== '') {
        $v[] = RPRESS_BRIDGE_PLUGIN;
    }
    if (defined('RPRESS_BRIDGE_PLUGINS')) {
        foreach ((array) json_decode(RPRESS_BRIDGE_PLUGINS, true) as $p) {
            if (is_string($p) && $p !== '') {
                $v[] = $p;
            }
        }
    }
    return $v;
});

/* get_bloginfo('version') reads raw $wp_version (filter `bloginfo` skipped).
   Elementor 4.3.5 requires WP 6.8; vendored core is 6.7.11. GDLR ignores this. */
if (isset($GLOBALS['wp_version'])) {
    $GLOBALS['wp_version'] = '6.8.3';
}

add_action('elementor/init', static function () {
    if (class_exists('\Elementor\Core\Kits\Manager')) {
        \Elementor\Core\Kits\Manager::create_default_kit();
    }
}, 0);

/* Frontend (`elementor-preview`) never hits admin_init / auth_redirect.
   Guest WP_Query 404s the iframe → editor stays on LOADING. Token already
   passed; uid 1 on every request. GDLR ajax still sets the same user. */
add_action('init', static function () {
    if (function_exists('wp_set_current_user')) {
        wp_set_current_user(1);
    }
}, 0);

add_filter('user_has_cap', static function ($caps) {
    $caps['edit_posts'] = true;
    $caps['edit_pages'] = true;
    $caps['edit_others_posts'] = true;
    $caps['edit_others_pages'] = true;
    $caps['edit_published_pages'] = true;
    $caps['edit_published_posts'] = true;
    return $caps;
}, 0);

/* CPT-cap grants live in role rows (wp_user_roles), which the bridge's
   fresh wp_install seeds without plugin caps. Activation hooks are
   unreliable under WP_INSTALLING (symlinked __FILE__ + transient races),
   so grant the tour/room/car/coupon/service caps tourmaster gives
   administrator directly — cheap, idempotent, request-local. */
add_filter('user_has_cap', static function ($caps) {
    foreach (['tour', 'room', 'car', 'coupon', 'service'] as $pt) {
        foreach (['edit_%s', 'read_%s', 'delete_%s', 'edit_%ss',
                  'edit_others_%ss', 'publish_%ss', 'read_private_%ss',
                  'delete_%ss'] as $c) {
            $caps[str_replace('%s', $pt, $c)] = true;
        }
    }
    foreach (['manage_tour_category', 'manage_tour_tag', 'manage_tour_filter',
              'manage_tour_order', 'manage_room_category', 'manage_room_tag',
              'manage_room_filter', 'manage_room_order'] as $mc) {
        $caps[$mc] = true;
    }
    return $caps;
}, 0);

/* WP 6.8 defaults every post type to the block editor: post.php mounts
   an empty React shell whose metabox region needs REST context the
   bridge sandbox can't answer, leaving a blank page. The bridged
   editor IS the GDLR/tourmaster metaboxes — classic layout always. */
add_filter('use_block_editor_for_post', '__return_false');
add_filter('use_block_editor_for_post_type', '__return_false');
/* The block editor's enqueued JS still runs (gutenberg screen ids),
   flips `is-fullscreen-mode`+`block-editor-page` and collapses #wpbody —
   0-height metaboxes, blank page. CSS can't trust class state (JS
   toggles after load); strip the classes and force #wpbody visible. */
add_action('admin_footer', static function () {
    if (get_current_screen()?->base !== 'post') { return; }
    echo '<script>document.addEventListener("DOMContentLoaded",function(){'
        . 'document.body.classList.remove("is-fullscreen-mode","block-editor-page");'
        . 'var w=document.getElementById("wpbody-content");'
        . 'if(w){w.style.display="";w.style.height="";w.style.visibility="";}'
        . 'document.querySelectorAll(".hide-if-js").forEach(function(e){e.style.display="none";});'
        . '});</script>';
    echo '<style>.block-editor-page #wpbody-content{display:block!important;}'
        . '.is-fullscreen-mode #wpbody{position:static!important;margin:0!important;}'
        . '.block-editor__container,#editor{display:none!important;}</style>';
}, 99);

add_action('template_redirect', static function () {
    if (empty($_GET['elementor-preview'])) {
        return;
    }
    if (function_exists('wp_set_current_user')) {
        wp_set_current_user(1);
    }
    $line = wp_json_encode([
        'uid' => get_current_user_id(),
        'the_id' => get_the_ID(),
        'get' => $_GET['elementor-preview'],
        'admin' => is_admin(),
        'can' => current_user_can('edit_post', (int) $_GET['elementor-preview']),
    ]);
    if (defined('FQDB')) {
        @file_put_contents(dirname(FQDB) . '/preview-debug.json', $line);
    }
}, 0);

/* Tevily page.php uses GVA_Layout_Frontend, never `the_content`, so
   Elementor's builder_wrapper never injects `.elementor-edit-area` and
   the editor iframe stays on LOADING. Force the classic loop for preview. */
add_filter('template_include', static function ($template) {
    if (empty($_GET['elementor-preview']) || !defined('FQDB')) {
        return $template;
    }
    $file = dirname(FQDB) . '/elementor-preview.php';
    if (!is_file($file)) {
        @file_put_contents(
            $file,
            "<?php\nget_header();\nif (have_posts()) { the_post(); the_content(); }\nget_footer();\n"
        );
    }
    return is_file($file) ? $file : $template;
}, 99);

add_action('elementor/editor/footer', static function () {
    echo <<<'JS'
<script>
(function () {
  function hideLoader() {
    var l = document.getElementById('elementor-loading');
    if (l) l.style.display = 'none';
    var w = document.querySelector('.elementor-loader-wrapper');
    if (w) w.style.display = 'none';
  }
  function iframeReady() {
    var f = document.getElementById('elementor-preview-iframe');
    try {
      return !!(f && f.contentDocument && f.contentDocument.querySelector('[data-elementor-id], .elementor'));
    } catch (e) { return false; }
  }
  var n = 0;
  var t = setInterval(function () {
    n += 1;
    if (iframeReady() || n > 15) { hideLoader(); clearInterval(t); }
  }, 1000);
})();
</script>
JS;
});

add_action('wp_enqueue_scripts', static function () {
    if (empty($_GET['elementor-preview'])) {
        return;
    }
    // Elementor preview enqueue does jQuery.holdReady(true) until the parent
    // editor releases it. Parent attach can't reach iframe jQuery → deadlock,
    // canvas spinner forever. Release ready on the preview document itself.
    wp_add_inline_script('jquery-migrate', 'if (window.jQuery) { jQuery.holdReady(false); }', 'after');
}, 100);

/* Identity: admin requests run as uid 1 (see header note 2). Also mint a
 * real nonce for gdlr ajax calls — our admin page can't hold a WP nonce,
 * and `check_ajax_referer('gdlr_core_page_builder')` would die('-1').
 * Runs after the token fence, so nonce-fakeability is bounded by the token. */
add_action('admin_init', function () {
    wp_set_current_user(1);
    if (!empty($_POST['action']) && $_POST['action'] === 'elementor_ajax') {
        $n = wp_create_nonce('elementor_ajax');
        $_POST['_nonce'] = $n;
        $_REQUEST['_nonce'] = $n;
    }
    if (!empty($_POST['action']) && strpos($_POST['action'], 'gdlr_core_') === 0) {
        $n = wp_create_nonce('gdlr_core_page_builder');
        $_POST['security'] = $n;
        $_REQUEST['security'] = $n;
    }
    /* `gdlr_core_save_page_option_data_<post_type>` checks the
       `gdlr_core_page_option` nonce (page-option.php:146) — mint it for
       both `security` and `page_option_security` so either code path
       (check_ajax_referer or wp_verify_nonce at :199) passes. */
    if (!empty($_POST['action']) && strpos($_POST['action'], 'gdlr_core_save_page_option_data') === 0) {
        $n = wp_create_nonce('gdlr_core_page_option');
        $_POST['security'] = $n;
        $_REQUEST['security'] = $n;
        $_POST['page_option_security'] = $n;
        $_REQUEST['page_option_security'] = $n;
    }
    /* html-option actions (`gdlr_get_gallery_options`, image fields inside
       element lightboxes) check the `gdlr_core_html_option` nonce — same
       mint path, different action name. */
    if (!empty($_POST['action']) && strpos($_POST['action'], 'gdlr_get_') === 0) {
        $n = wp_create_nonce('gdlr_core_html_option');
        $_POST['security'] = $n;
        $_REQUEST['security'] = $n;
    }
    /* `upload-attachment` + `media-create-image-subsizes` check
       check_ajax_referer('media-form') — library browsing doesn't, but
       bridge uploads 403 without it. Mint for those two actions. */
    if (!empty($_POST['action']) && in_array($_POST['action'], ['upload-attachment', 'media-create-image-subsizes'], true)) {
        $n = wp_create_nonce('media-form');
        $_POST['_wpnonce'] = $n;
        $_REQUEST['_wpnonce'] = $n;
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
        $pid = 0;
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
            }
        } else if (is_array($p) && !empty($p['ID'])) {
            /* Post row already exists (skeleton wp.db carries it across
               spawns) — rpress.db is truth, so refresh row + meta or a
               stale skel resurrects pre-edit content over the pull. */
            $pid = (int) $p['ID'];
            wp_update_post([
                'ID'           => $pid,
                'post_title'   => $p['post_title'] ?? '',
                'post_content' => $p['post_content'] ?? '',
                'post_excerpt' => $p['post_excerpt'] ?? '',
                'post_status'  => $p['post_status'] ?? 'draft',
                'post_name'    => $p['post_name'] ?? '',
                'post_type'    => $p['post_type'] ?? 'page',
            ]);
        }
        if (!empty($pid) && !is_wp_error($pid)
            && !empty($j['meta']) && is_array($j['meta'])) {
            global $wpdb;
            /* Meta rows seeded byte-exact: write via $wpdb directly.
               update_post_meta's maybe_serialize/wp_unslash pipeline
               double-wraps already-serialized values (observed GDLR
               s:N:"s:N:"a:19:..." compounding on every respawn+pull). */
            foreach ($j['meta'] as $mk => $mv) {
                $mid = $wpdb->get_var($wpdb->prepare(
                    'SELECT meta_id FROM ' . $wpdb->postmeta . ' WHERE post_id=%d AND meta_key=%s',
                    $pid, $mk
                ));
                if ($mid) {
                    $wpdb->update($wpdb->postmeta, ['meta_value' => $mv], ['meta_id' => $mid]);
                } else {
                    $wpdb->insert($wpdb->postmeta,
                        ['post_id' => $pid, 'meta_key' => $mk, 'meta_value' => $mv]);
                }
            }
            /* Cheap corruption alarm: if _elementor_data doesn't
               decode right after seeding, the editor would silently
               rebuild a stub layout — make it loud in the bridge log. */
            $ed = get_post_meta($pid, '_elementor_data', true);
            if ($ed !== '' && json_decode($ed) === null) {
                error_log('rpress seed: _elementor_data is not valid JSON for post ' . $pid);
            }
        }
        if (!empty($j['attachments']) && is_array($j['attachments'])) {
            /* Attachment library rows for query-attachments — idempotent
               upsert keyed on wp_posts ID, same byte-exact $wpdb path. */
            foreach ($j['attachments'] as $a) {
                $aid = (int) ($a['ID'] ?? 0);
                $afields = [
                    'post_title'   => $a['post_title'] ?? '',
                    'post_content' => $a['post_content'] ?? '',
                    'post_excerpt' => $a['post_excerpt'] ?? '',
                    'post_status'  => 'inherit',
                    'post_name'    => $a['post_name'] ?? '',
                    'post_parent'  => 0,
                    'post_type'    => 'attachment',
                    'post_mime_type' => $a['post_mime_type'] ?? '',
                    'guid'         => rtrim(get_option('siteurl'), '/') . '/wp-content/uploads/' . ltrim($a['guid'] ?? '', '/'),
                ];
                if ($aid && get_post($aid)) {
                    $wpdb->update($wpdb->posts, $afields, ['ID' => $aid]);
                } elseif ($aid) {
                    $wpdb->insert($wpdb->posts, $afields + [
                        'ID' => $aid,
                        'post_date' => $a['post_date'] ?? '0000-00-00 00:00:00',
                        'post_date_gmt' => $a['post_date'] ?? '0000-00-00 00:00:00',
                        'post_modified' => $a['post_modified'] ?? '0000-00-00 00:00:00',
                        'post_modified_gmt' => $a['post_modified'] ?? '0000-00-00 00:00:00',
                    ]);
                }
                $am = $j['attachment_meta'][$aid] ?? null;
                if ($aid && is_array($am)) {
                    foreach ($am as $ak => $av) {
                        $amid = $wpdb->get_var($wpdb->prepare(
                            'SELECT meta_id FROM ' . $wpdb->postmeta . ' WHERE post_id=%d AND meta_key=%s',
                            $aid, $ak
                        ));
                        if ($amid) {
                            $wpdb->update($wpdb->postmeta, ['meta_value' => $av], ['meta_id' => $amid]);
                        } else {
                            $wpdb->insert($wpdb->postmeta,
                                ['post_id' => $aid, 'meta_key' => $ak, 'meta_value' => $av]);
                        }
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
