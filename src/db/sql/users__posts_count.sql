SELECT COUNT(*) FROM posts p
JOIN post_meta m ON m.post_id=p.id AND m.meta_key='author_id'
WHERE p.status='publish' AND m.meta_value=?1