SELECT p.id, COALESCE(p.title,''), COALESCE(p.modified_at,'')
FROM posts p JOIN post_meta m ON m.post_id=p.id AND m.meta_key='author_id'
WHERE p.status='draft' AND p.kind='post' AND m.meta_value=?1
ORDER BY p.modified_at DESC, p.id DESC
LIMIT ?2
