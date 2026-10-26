SELECT meta_value FROM post_meta
             WHERE post_id=?1 AND meta_key='_wp_old_slug'
             ORDER BY rowid