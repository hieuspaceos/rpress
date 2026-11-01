SELECT COALESCE(p.wp_id, m.post_id), m.meta_key, m.meta_value
FROM post_meta m
JOIN posts p ON p.id = m.post_id
WHERE p.status='publish'
ORDER BY COALESCE(p.wp_id, m.post_id)
