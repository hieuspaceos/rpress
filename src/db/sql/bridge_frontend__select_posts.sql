SELECT COALESCE(wp_id, id), title, body_html, excerpt, status,
       slug, kind, COALESCE(parent_id,0), COALESCE(menu_order,0),
       COALESCE(published_at,''), COALESCE(modified_at,''), featured_media_id
FROM posts WHERE status='publish'
ORDER BY COALESCE(wp_id, id)
