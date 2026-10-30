SELECT COALESCE(wp_id, id), title, body_html, excerpt, status,
       slug, kind, COALESCE(parent_id,0), COALESCE(menu_order,0),
       COALESCE(published_at,''), COALESCE(modified_at,'')
FROM posts WHERE id = ?1
