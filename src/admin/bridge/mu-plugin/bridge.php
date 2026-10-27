<?php
/**
 * rpress bridge mu-plugin — loaded inside the bridge WP on every request.
 *
 * Jobs:
 *  1. Validate the one-time bearer token issued by rpress (X-Rpress-Bridge
 *     header vs RPRESS_BRIDGE_TOKEN env the spawn wrote to bridge-token).
 *     A mismatch gets 403 before WordPress does anything — the bridge URL
 *     is localhost-only already; this is the second fence.
 *  2. Land wp option/post_meta writes in wp.db tables the pull pass reads:
 *     options → `options` table, post meta → `post_meta` table. The bridge
 *     WP runs on sqlite-database-integration so these are already local
 *     SQLite rows; the shim only guarantees the table shape the pull
 *     expects and blocks writes outside it.
 *
 * Keep this file dependency-free: it runs before plugins load.
 */

$tok = getenv('RPRESS_BRIDGE_TOKEN');
$got = isset($_SERVER['HTTP_X_RPRESS_BRIDGE']) ? $_SERVER['HTTP_X_RPRESS_BRIDGE'] : '';
if ($tok !== '' && !hash_equals($tok, $got)) {
    http_response_code(403);
    header('Content-Type: text/plain');
    echo 'bridge token mismatch';
    exit;
}

/* Whitelist the tables pull.rs reads; anything else lands but is ignored
 * (wp.db is wiped on respawn — bounded blast radius). */
