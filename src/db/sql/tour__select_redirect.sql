SELECT new_url FROM redirects
             WHERE old_slug=?2 AND source='old-slug'
             ORDER BY rowid
             LIMIT 1 OFFSET (SELECT COUNT(DISTINCT m.post_id) FROM post_meta m
                 WHERE m.meta_key='_wp_old_slug' AND m.meta_value=?2
                 AND m.rowid < (SELECT MIN(rowid) FROM post_meta
                     WHERE post_id=?1 AND meta_key='_wp_old_slug'))