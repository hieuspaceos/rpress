SELECT p.wp_id FROM redirects r
             JOIN posts p ON '/'||p.slug||'/' = r.new_url
             WHERE r.old_slug = ?1 AND p.status='publish'
             ORDER BY p.id LIMIT 1