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

    /* Frontend slug passes an empty RPRESS_BRIDGE_PLUGIN — only there does
       the admin bar pollute origin parity. */
    if (defined('RPRESS_BRIDGE_PLUGIN') && RPRESS_BRIDGE_PLUGIN === '') {
        add_filter('show_admin_bar', '__return_false');
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
/* Frontend bridge seed (site.json): all posts + meta + attachments +
   nav menus + theme locations + the option subset single.php needs.
   Runs on `init` (frontend requests never hit admin_init) only when
   the file exists — admin bridges keep their post.json path. Every
   insert is $wpdb-direct / byte-exact; wp_* helpers would double-wrap
   serialized meta. Guard: option `rpress_site_seeded`. */
add_action('init', static function () {
    $seed = WP_CONTENT_DIR . '/rpress-seed/site.json';
    if (!is_file($seed) || get_option('rpress_site_seeded')) {
        return;
    }
    $j = json_decode(file_get_contents($seed), true);
    if (!is_array($j)) {
        return;
    }
    global $wpdb;
    update_option('rpress_site_seeded', 1, false);

    error_log('rpress frontend seed: starting site.json import');

    $opts = $j['options'] ?? [];
    if (is_array($opts)) {
        foreach ($opts as $k => $v) {
            /* Options carry PHP-serialized values (theme_mods_*, nav_menu_locations).
               Passing the raw string through add_option double-serializes →
               get_option returns array(0=>'a:5:{...}') and has_nav_menu('primary')
               is false (observed: mobile menu empty on blueskies). */
            if (is_string($v)) {
                $v2 = @unserialize($v);
                if ($v2 !== false) { $v = $v2; }
            }
            delete_option($k);
            add_option($k, $v, '', 'no');
        }
    }

    /* Origin wp_options merge (options.json): plugin/theme render state —
       elementor kit, trustindex connection keys, redux, cf7. Bridge-owned
       keys are excluded so seeded siteurl/active_plugins/rpress flags and
       WP-internal constants aren't clobbered. site.json `options` (above)
       still wins — it carries curated overrides. */
    $skip = ['siteurl','home','active_plugins','blogname','template','stylesheet',
             'rpress_site_seeded','rpress_post_seeded','rpress_seed_done',
             'cron','nonce_key','nonce_salt','auth_key','auth_salt',
             'logged_in_key','logged_in_salt','secure_auth_key','secure_auth_salt',
             'admin_email','new_admin_email','users_can_register','default_role',
             'fresh_site','initial_db_version','site_created_time','siteurl_old',
             'recovery_keys','recovery_mode_email_last_sent'];
    $raw_opts = @json_decode(file_get_contents(WP_CONTENT_DIR . '/rpress-seed/options.json'), true);
    if (is_array($raw_opts)) {
        foreach ($raw_opts as $k => $v) {
            if (in_array($k, $skip, true) || strpos($k, '_transient') !== false || strpos($k, '_site_transient') !== false) {
                continue;
            }
            if (is_string($v)) {
                $v2 = @unserialize($v);
                if ($v2 !== false) { $v = $v2; }
            }
            if (!isset($opts[$k])) {
                delete_option($k);
                add_option($k, $v, '', 'no');
            }
        }
    }

    $upsert_post = static function (array $p, string $type = '', string $guid = '') use ($wpdb): int {
        $pid = (int) ($p['ID'] ?? 0);
        if (!$pid) {
            return 0;
        }
        $row = [
            'post_author' => 1,
            'post_date' => $p['post_date'] ?? '0000-00-00 00:00:00',
            'post_date_gmt' => $p['post_date'] ?? '0000-00-00 00:00:00',
            'post_content' => $p['post_content'] ?? '',
            'post_title' => $p['post_title'] ?? '',
            'post_excerpt' => $p['post_excerpt'] ?? '',
            'post_status' => $p['post_status'] ?? 'publish',
            'comment_status' => 'closed',
            'ping_status' => 'closed',
            'post_name' => $p['post_name'] ?? '',
            'post_modified' => $p['post_modified'] ?? '0000-00-00 00:00:00',
            'post_modified_gmt' => $p['post_modified'] ?? '0000-00-00 00:00:00',
            'post_parent' => (int) ($p['post_parent'] ?? 0),
            'guid' => $guid,
            'menu_order' => (int) ($p['menu_order'] ?? 0),
            'post_type' => $type !== '' ? $type : ($p['post_type'] ?? 'page'),
            'post_mime_type' => $p['post_mime_type'] ?? '',
            'comment_count' => 0,
        ];
        if ($wpdb->get_var($wpdb->prepare("SELECT ID FROM {$wpdb->posts} WHERE ID=%d", $pid))) {
            $wpdb->update($wpdb->posts, $row, ['ID' => $pid]);
        } else {
            $row['ID'] = $pid;
            $wpdb->insert($wpdb->posts, $row);
        }
        return $pid;
    };
    $upsert_meta = static function (int $pid, string $key, string $val) use ($wpdb): void {
        $mid = $wpdb->get_var($wpdb->prepare(
            "SELECT meta_id FROM {$wpdb->postmeta} WHERE post_id=%d AND meta_key=%s", $pid, $key));
        if ($mid) {
            $wpdb->update($wpdb->postmeta, ['meta_value' => $val], ['meta_id' => $mid]);
        } else {
            $wpdb->insert($wpdb->postmeta, ['post_id' => $pid, 'meta_key' => $key, 'meta_value' => $val]);
        }
    };

    foreach ((array) ($j['posts'] ?? []) as $p) {
        if (!is_array($p)) { continue; }
        $pid = $upsert_post($p, '', '');
        $fid = (int) ($p['_thumbnail_id'] ?? 0);
        if ($pid && $fid) {
            $upsert_meta($pid, '_thumbnail_id', (string) $fid);
        }
    }
    /* Cached Elementor render fragments from the live site were built
       with a different plugin context — they container-wrap and DROP
       unregistered widget types (observed: gva_post_* / gva-template_content
       stripped on blueskies). Skip the key; Elementor re-caches per bridge. */
    $meta = $j['meta'] ?? [];
    if (is_array($meta)) {
        foreach ($meta as $pid => $m) {
            if (is_array($m)) {
                foreach ($m as $k => $v) {
                    if ($k === '_elementor_element_cache') { continue; }
                    $upsert_meta((int) $pid, (string) $k, (string) $v);
                }
            }
        }
    }

    $site_url = rtrim((string) get_option('siteurl'), '/');
    $am = $j['attachment_meta'] ?? [];
    foreach ((array) ($j['attachments'] ?? []) as $a) {
        if (!is_array($a)) { continue; }
        $rel = ltrim((string) ($a['guid'] ?? ''), '/');
        $guid = $site_url . '/wp-content/uploads/' . $rel;
        $a['post_type'] = 'attachment';
        $a['post_status'] = 'inherit';
        $aid = $upsert_post($a, 'attachment', $guid);
        if ($aid && isset($am[(string) $aid]) && is_array($am[(string) $aid])) {
            foreach ($am[(string) $aid] as $k => $v) {
                $upsert_meta($aid, (string) $k, (string) $v);
            }
        }
    }

    /* Dump-sourced CPTs rpress doesn't model (gva__template header/footer/
       single layouts, nav_menu_item, elementor_library). Same byte-exact
       upsert; meta rides the shared meta map under their wp_id. */
    foreach ((array) ($j['extra_posts'] ?? []) as $p) {
        if (!is_array($p)) { continue; }
        $pid = $upsert_post($p, '', (string) ($p['guid'] ?? ''));
        if ($pid && !empty($p['_thumbnail_id'])) {
            $upsert_meta($pid, '_thumbnail_id', (string) (int) $p['_thumbnail_id']);
        }
    }

    /* nav menus: term rows + nav_menu taxonomy + nav_menu_item posts +
       menu-item meta; theme_locations become theme_mods.nav_menu_locations
       (handled via the seeded option below). */
    /* Terms + taxonomy: wp.db term ids don't exist in rpress.db — assign
       via insert_id, keyed by (taxonomy, slug) for relationships. Menus
       (nav_menu) are seeded below with fixed wp_term_id; skip dupes by
       slug so a menu term doesn't get two term_taxonomy rows. */
    $ttid_map = [];
    /* Menu terms must claim their wp_term_id FIRST — the generic terms
       loop assigns auto-increment ids that can collide (blueskies: a
       taxonomies_list 'features' row grabbed term_id 8, and the nav_menu
       term then never got created → empty menus). */
    foreach ((array) ($j['menus'] ?? []) as $m) {
        if (!is_array($m)) { continue; }
        $tid = (int) ($m['wp_term_id'] ?? 0);
        if (!$tid) { continue; }
        if (!$wpdb->get_var($wpdb->prepare("SELECT term_id FROM {$wpdb->terms} WHERE term_id=%d", $tid))) {
            $wpdb->insert($wpdb->terms, ['term_id' => $tid, 'name' => (string) ($m['name'] ?? 'menu'), 'slug' => 'rpress-menu-' . $tid]);
        }
    }
    foreach ((array) ($j['terms'] ?? []) as $t) {
        if (!is_array($t)) { continue; }
        $tax = (string) ($t['taxonomy'] ?? '');
        $slug = (string) ($t['slug'] ?? '');
        if ($tax === '' || $slug === '' || $tax === 'nav_menu') { continue; }
        $key = $tax . ':' . $slug;
        $existing = $wpdb->get_row($wpdb->prepare(
            "SELECT t.term_id, tt.term_taxonomy_id FROM {$wpdb->terms} t
             JOIN {$wpdb->term_taxonomy} tt ON tt.term_id = t.term_id
             WHERE t.slug=%s AND tt.taxonomy=%s", $slug, $tax));
        if ($existing) {
            $ttid_map[$key] = (int) $existing->term_taxonomy_id;
            continue;
        }
        $wpdb->insert($wpdb->terms, ['name' => (string) ($t['name'] ?? $slug), 'slug' => $slug]);
        $term_id = (int) $wpdb->insert_id;
        if (!$term_id) { continue; }
        $wpdb->insert($wpdb->term_taxonomy, [
            'term_id' => $term_id, 'taxonomy' => $tax,
            'description' => (string) ($t['description'] ?? ''), 'parent' => 0, 'count' => 0,
        ]);
        $ttid_map[$key] = (int) $wpdb->insert_id;
    }
    /* Hierarchy: parent_slug resolves to a term already seeded above. */
    foreach ((array) ($j['terms'] ?? []) as $t) {
        if (!is_array($t)) { continue; }
        $tax = (string) ($t['taxonomy'] ?? '');
        $pslug = (string) ($t['parent_slug'] ?? '');
        if ($tax === '' || $pslug === '') { continue; }
        $child_ttid = $ttid_map[$tax . ':' . ($t['slug'] ?? '')] ?? 0;
        $parent_ttid = $ttid_map[$tax . ':' . $pslug] ?? 0;
        if (!$child_ttid || !$parent_ttid) { continue; }
        $parent_term_id = (int) $wpdb->get_var($wpdb->prepare(
            "SELECT term_id FROM {$wpdb->term_taxonomy} WHERE term_taxonomy_id=%d", $parent_ttid));
        if ($parent_term_id) {
            $wpdb->update($wpdb->term_taxonomy, ['parent' => $parent_term_id], ['term_taxonomy_id' => $child_ttid]);
        }
    }
    foreach ((array) ($j['post_terms'] ?? []) as $pt) {
        if (!is_array($pt)) { continue; }
        $wpid = (int) ($pt['post_wp_id'] ?? 0);
        $ttid = $ttid_map[($pt['taxonomy'] ?? '') . ':' . ($pt['slug'] ?? '')] ?? 0;
        if (!$wpid || !$ttid) { continue; }
        if (!$wpdb->get_var($wpdb->prepare(
            "SELECT object_id FROM {$wpdb->term_relationships} WHERE object_id=%d AND term_taxonomy_id=%d", $wpid, $ttid))) {
            $wpdb->insert($wpdb->term_relationships, [
                'object_id' => $wpid, 'term_taxonomy_id' => $ttid, 'term_order' => 0,
            ]);
            $wpdb->query($wpdb->prepare(
                "UPDATE {$wpdb->term_taxonomy} SET count=count+1 WHERE term_taxonomy_id=%d", $ttid));
        }
    }

    foreach ((array) ($j['menus'] ?? []) as $m) {
        if (!is_array($m)) { continue; }
        $tid = (int) ($m['wp_term_id'] ?? 0);
        $name = (string) ($m['name'] ?? '');
        if (!$tid || $name === '') { continue; }
        /* Widgets call wp_nav_menu(['menu' => 'main-menu']) — the slug must
           be the WP menu slug (rpress menus.location), else core falls back
           to wp_page_menu and links render empty (observed on blueskies). */
        $mslug = (string) ($m['location'] ?? '');
        if ($mslug === '') { $mslug = 'menu-' . $tid; }
        if (!$wpdb->get_var($wpdb->prepare("SELECT term_id FROM {$wpdb->terms} WHERE term_id=%d", $tid))) {
            $wpdb->insert($wpdb->terms, ['term_id' => $tid, 'name' => $name, 'slug' => $mslug]);
        } else {
            $wpdb->update($wpdb->terms, ['name' => $name, 'slug' => $mslug], ['term_id' => $tid]);
        }
        if (!$wpdb->get_var($wpdb->prepare(
            "SELECT term_taxonomy_id FROM {$wpdb->term_taxonomy} WHERE term_id=%d AND taxonomy='nav_menu'", $tid))) {
            $wpdb->insert($wpdb->term_taxonomy, [
                'term_id' => $tid, 'taxonomy' => 'nav_menu', 'description' => '', 'parent' => 0, 'count' => 0,
            ]);
        }
    }
    foreach ((array) ($j['menu_items'] ?? []) as $mi) {
        if (!is_array($mi)) { continue; }
        $iid = (int) ($mi['ID'] ?? 0);
        if (!$iid) { continue; }
        $mi['post_type'] = 'nav_menu_item';
        $mi['post_status'] = 'publish';
        $mi['post_name'] = 'menu-item-' . $iid;
        $mi['post_title'] = (string) ($mi['label'] ?? '');
        $mi['post_parent'] = 0;
        $mi['menu_order'] = (int) ($mi['ord'] ?? 0);
        $upsert_post($mi, 'nav_menu_item', '');
        $type = (string) ($mi['item_type'] ?? 'custom');
        $upsert_meta($iid, '_menu_item_type', $type === 'post_type' ? 'post_type' : 'custom');
        $upsert_meta($iid, '_menu_item_menu_item_parent', (string) (int) ($mi['parent_id'] ?? 0));
        $upsert_meta($iid, '_menu_item_object_id', (string) (int) ($mi['object_id'] ?? 0));
        $upsert_meta($iid, '_menu_item_object', 'page');
        $upsert_meta($iid, '_menu_item_url', (string) ($mi['url'] ?? ''));
        $upsert_meta($iid, '_menu_item_target', (string) ($mi['target'] ?? ''));
        $upsert_meta($iid, '_menu_item_classes', serialize([(string) ($mi['classes'] ?? '')]));
        $upsert_meta($iid, '_menu_item_xfn', (string) ($mi['xfn'] ?? ''));
        $term_id = (int) ($mi['menu_term_id'] ?? 0);
        if ($term_id) {
            $tt = $wpdb->get_var($wpdb->prepare(
                "SELECT term_taxonomy_id FROM {$wpdb->term_taxonomy} WHERE term_id=%d AND taxonomy='nav_menu'", $term_id));
            if ($tt && !$wpdb->get_var($wpdb->prepare(
                "SELECT object_id FROM {$wpdb->term_relationships} WHERE object_id=%d AND term_taxonomy_id=%d", $iid, $tt))) {
                $wpdb->insert($wpdb->term_relationships, [
                    'object_id' => $iid, 'term_taxonomy_id' => $tt, 'term_order' => (int) ($mi['ord'] ?? 0),
                ]);
            }
        }
    }

    /* wp_get_nav_menu_items short-circuits to [] when term_taxonomy.count
       is 0 — recount after relationships land. */
    $wpdb->query("UPDATE {$wpdb->term_taxonomy} tt SET count=(SELECT COUNT(*) FROM {$wpdb->term_relationships} tr WHERE tr.term_taxonomy_id=tt.term_taxonomy_id) WHERE tt.taxonomy='nav_menu'");

    /* nav_menu_locations: `menus.theme_location` is the WP location slug —
       the option WP reads is theme_mods_{stylesheet}.nav_menu_locations. */
    $locs = [];
    foreach ((array) ($j['menus'] ?? []) as $m) {
        $loc = (string) ($m['theme_location'] ?? '');
        $tid = (int) ($m['wp_term_id'] ?? 0);
        if ($loc !== '' && $tid) {
            $locs[$loc] = $tid;
        }
    }
    if ($locs) {
        $sheet = (string) get_option('stylesheet');
        $key = 'theme_mods_' . $sheet;
        $mods = (array) get_option($key, []);
        $mods['nav_menu_locations'] = $locs;
        update_option($key, $mods);
    }
    /* _elementor_css meta says status=file but the generated
       post-N.css was never imported — links 404 → unstyled widgets
       (observed: grey BOOK NOW buttons). Regenerate once per bridge;
       files land in the shared uploads dir the static server serves.
       MUST run after the meta seed above — pids come from _elementor_css. */
    if (class_exists('Elementor\Core\Files\CSS\Post') && !get_option('rpress_css_seeded')) {
        update_option('rpress_css_seeded', 1, false);
        $css_dir = wp_upload_dir()['basedir'] . '/elementor/css';
        wp_mkdir_p($css_dir);
        $pids = $wpdb->get_col("SELECT post_id FROM {$wpdb->postmeta} WHERE meta_key='_elementor_css'");
        foreach ($pids as $cpid) {
            $f = $css_dir . '/post-' . $cpid . '.css';
            if (file_exists($f)) { continue; }
            try {
                (new Elementor\Core\Files\CSS\Post((int) $cpid))->update();
            } catch (\Throwable $e) {
                error_log('rpress css regen ' . $cpid . ': ' . $e->getMessage());
            }
        }
    }

}, 5);


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
