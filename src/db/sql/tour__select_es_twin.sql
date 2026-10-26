SELECT p2.slug FROM posts p2
             WHERE p2.kind='tour' AND p2.lang='es' AND p2.status='publish'
               AND p2.menu_order=(SELECT menu_order FROM posts WHERE id=?1)
               AND p2.featured_media_id=(SELECT featured_media_id FROM posts WHERE id=?1)
             LIMIT 1